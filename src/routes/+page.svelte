<script lang="ts">
  import { onMount } from 'svelte';
  import { fade, fly } from 'svelte/transition';
  import { 
    Layers, Activity, Box, Settings2, Target, Hash, Info, ChevronLeft, ChevronRight, Menu 
  } from 'lucide-svelte';
  import PartialSum2D from '$lib/components/PartialSum2D.svelte';
  import PartialSum3D from '$lib/components/PartialSum3D.svelte';
  import DomainColoring from '$lib/components/DomainColoring.svelte';
  import { zeta, calculateZetaPath } from '$lib/math/zeta';
  import { Complex } from '$lib/math/complex';

  let sigma = $state(0.5);
  let t = $state(14.134725);
  let N = $state(100);
  let currentView = $state('sum3d');
  let currentMode = $state('zeta');
  let s = $derived(Complex.from(sigma, t));

  // Simulation state
  let infiniteSum = $state<Complex>(new Complex(0, 0));
  let zetaTrajectory = $state<{re: number, im: number, t: number}[]>([]);
  let analyticalCrossings = $state<{t: number, im: number, isZero: boolean}[]>([]);
  let analyticalTMin = $state(10);
  let analyticalTMax = $state(20);
  let calculatingPath = $state(false);
  let discoveredZeros = $state<number[]>([]);

  $effect(() => {
    if (N > 50000) N = 50000;
  });

  $effect(() => {
    infiniteSum = zeta(s, Math.min(N, 1000));
  });

  function setView(view: string) {
    currentView = view;
  }

  function applyPreset(p: any) {
    sigma = p.sigma;
    t = p.t;
    N = p.N;
    if (p.mode) currentMode = p.mode;
  }

  const presets = [
    { name: 'Critical Zero 1', sigma: 0.5, t: 14.134725, N: 1000 },
    { name: 'Critical Zero 2', sigma: 0.5, t: 21.022040, N: 1000 },
    { name: '10k Term Spiral', sigma: 0.5, t: 37.0, N: 10000 },
    { name: 'Euler Convergent', sigma: 2.0, t: 5.0, N: 1000, mode: 'euler' },
    { name: 'Moebius Chaotic', sigma: 0.5, t: 30.0, N: 1000, mode: 'reciprocal' }
  ];

  let sidebarVisible = $state(true);

  async function computeFullPath() {
    calculatingPath = true;
    try {
      const path = await calculateZetaPath(sigma, analyticalTMin, analyticalTMax, 5000);
      zetaTrajectory = path;
    } finally {
      calculatingPath = false;
    }
  }

  import { findAnalyticalCrossings } from '$lib/math/zeta';

  async function findZeros() {
    calculatingPath = true;
    try {
      const min = Math.min(analyticalTMin, analyticalTMax);
      const max = Math.max(analyticalTMin, analyticalTMax);
      const tCenter = (min + max) / 2;
      const range = (max - min) / 2;

      const crossings = await findAnalyticalCrossings(sigma, tCenter, range, 100000);
      
      const rawPoints = crossings
        .map(c => c.t)
        .filter(t => t >= min && t <= max)
        .sort((a, b) => a - b);
      
      // Clustering deduplication (more sensitive)
      const unique = [];
      if (rawPoints.length > 0) {
        unique.push(rawPoints[0]);
        for (let i = 1; i < rawPoints.length; i++) {
          if (rawPoints[i] - unique[unique.length - 1] > 0.01) {
            unique.push(rawPoints[i]);
          }
        }
      }
      discoveredZeros = unique;
    } finally {
      calculatingPath = false;
    }
  }

  function copyZeros() {
    const text = discoveredZeros.map(z => z.toFixed(8)).join('\n');
    navigator.clipboard.writeText(text);
  }
</script>

<main>
  <div class="background-glow"></div>

  <div class="sidebar glass" class:hidden={!sidebarVisible}>
    <!-- Integrated Toggle Button (Attached to the edge) -->
    <button 
      class="sidebar-toggle glass {sidebarVisible ? 'open' : ''}" 
      onclick={() => sidebarVisible = !sidebarVisible}
      aria-label={sidebarVisible ? 'Hide Settings' : 'Show Settings'}
    >
      {#if sidebarVisible}
        <ChevronLeft size={18} />
      {:else}
        <ChevronRight size={18} />
      {/if}
    </button>

    <header>
      <h1>Riemann <span class="gradient-text">Zeta</span></h1>
      <p class="subtitle">Mathematical Visualization Engine</p>
    </header>

    <div class="scroll-area">
      <nav>
        <div class="nav-group">
          <div class="nav-label">View Type</div>
          <div class="nav-row">
            <button class="icon-btn {currentView === 'all' ? 'active' : ''}" onclick={() => setView('all')} title="All Results">
              <Layers size={18} />
            </button>
            <button class="icon-btn {currentView === 'sum2d' ? 'active' : ''}" onclick={() => setView('sum2d')} title="2D Sum">
              <Activity size={18} />
            </button>
            <button class="icon-btn {currentView === 'sum3d' ? 'active' : ''}" onclick={() => setView('sum3d')} title="3D Sum">
              <Box size={18} />
            </button>
          </div>
        </div>

        <div class="nav-group">
          <div class="nav-label">Mathematical Form</div>
          <div class="mode-switcher-compact">
            <button class="mode-btn {currentMode === 'zeta' ? 'active' : ''}" onclick={() => currentMode = 'zeta'}>Zeta</button>
            <button class="mode-btn {currentMode === 'euler' ? 'active' : ''}" onclick={() => currentMode = 'euler'}>Euler</button>
            <button class="mode-btn {currentMode === 'reciprocal' ? 'active' : ''}" onclick={() => currentMode = 'reciprocal'}>Moebius</button>
          </div>
        </div>

        <div class="nav-group">
          <div class="nav-label">Parameters</div>
          <div class="control-row">
            <label class="control-header">
              <span>Real Part (σ)</span>
              <div class="input-slider-row">
                <input type="number" step="0.001" bind:value={sigma} class="num-input" />
                <input type="range" min="-5" max="5" step="0.001" bind:value={sigma} />
              </div>
            </label>
          </div>
          <div class="control-row">
            <label class="control-header">
              <span>Imaginary Part (t)</span>
              <div class="input-slider-row">
                <input type="number" step="0.0001" bind:value={t} class="num-input" />
                <input type="range" min="-1000" max="1000" step="0.0001" bind:value={t} />
              </div>
            </label>
          </div>
          <div class="control-row">
            <label class="control-header">
              <span>Iterations (N)</span>
              <div class="input-slider-row">
                <input type="number" step="1" bind:value={N} class="num-input" />
                <input type="range" min="1" max="50000" step="10" bind:value={N} />
              </div>
            </label>
          </div>

          {#if infiniteSum}
          <div class="value-display">
            <span class="label">Infinite Sum ζ(s)</span>
            <span class="value">{infiniteSum.r.toFixed(6)} {infiniteSum.i >= 0 ? '+' : ''}{infiniteSum.i.toFixed(6)}i</span>
          </div>
          {/if}
        </div>

        <div class="section-divider"></div>

        <div class="section-divider"></div>

        <div class="nav-group">
          <div class="nav-label">Analysis</div>
          <div class="nav-row" style="display: flex; gap: 6px; margin-bottom: 8px;">
            <button class="action-pill" style="flex: 1; min-width: 0;" onclick={() => zetaTrajectory = []} disabled={zetaTrajectory.length === 0}>
              Clear Path
            </button>
            <button class="action-pill" style="flex: 1; min-width: 0;" onclick={findZeros} disabled={calculatingPath}>
              {calculatingPath ? '...' : 'Print Zeros'}
            </button>
          </div>
          <div class="range-grid">
            <div class="input-wrap">
              <span>Start T</span>
              <input type="number" bind:value={analyticalTMin} />
            </div>
            <div class="input-wrap">
              <span>End T</span>
              <input type="number" bind:value={analyticalTMax} />
            </div>
            <button class="compute-btn {calculatingPath ? 'loading' : ''}" onclick={computeFullPath} disabled={calculatingPath}>
              {calculatingPath ? '...' : 'Compute'}
            </button>
          </div>
        </div>

        <div class="section-divider"></div>

        <div class="nav-group">
          <div class="nav-label">Presets</div>
          <div class="presets-row">
            {#each presets as p}
              <button class="preset-tag" onclick={() => applyPreset(p)}>{p.name}</button>
            {/each}
          </div>
        </div>
      </nav>
    </div>
  </div>

  <div class="viewport full-screen">
    {#if currentView === 'all'}
      <div class="view-container">
        <DomainColoring width={1000} height={1000} />
      </div>
    {:else if currentView === 'sum2d'}
      <div class="view-container">
        <PartialSum2D {s} {N} mode={currentMode} {infiniteSum} trajectory={zetaTrajectory} />
      </div>
    {:else if currentView === 'sum3d'}
      <div class="view-container">
        <PartialSum3D {s} {N} mode={currentMode} {infiniteSum} trajectory={zetaTrajectory} />
      </div>
    {/if}

    {#if discoveredZeros.length > 0}
      <div class="zeros-panel glass" transition:fly={{ y: 20, duration: 300 }}>
        <div class="panel-header">
          <span>Discovered Zeros</span>
          <button class="copy-btn" onclick={copyZeros}>Copy All</button>
        </div>
        <div class="zeros-list">
          {#each discoveredZeros as zero}
            <div class="zero-item">
              <span class="zero-val">t = {zero.toFixed(8)}</span>
              <button class="goto-btn" onclick={() => t = zero}>Go</button>
            </div>
          {/each}
        </div>
        <button class="close-panel" onclick={() => discoveredZeros = []}>×</button>
      </div>
    {/if}
  </div>
</main>

<style>
  .zeros-panel {
    position: absolute;
    bottom: 2rem;
    right: 2rem;
    width: 280px;
    max-height: 400px;
    display: flex;
    flex-direction: column;
    z-index: 100;
    padding: 1.25rem;
    animation: slideUp 0.3s ease-out;
  }

  .panel-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1rem;
    padding-bottom: 0.5rem;
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  }

  .panel-header span {
    font-size: 0.85rem;
    font-weight: 600;
    color: var(--accent-primary);
  }

  .copy-btn {
    background: rgba(34, 211, 238, 0.2);
    color: var(--accent-primary);
    border: 1px solid var(--accent-primary);
    padding: 2px 8px;
    font-size: 0.65rem;
    border-radius: 4px;
    cursor: pointer;
  }

  .zeros-list {
    max-height: 340px;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 6px;
    padding-right: 4px;
  }

  .zeros-list::-webkit-scrollbar {
    width: 4px;
  }

  .zeros-list::-webkit-scrollbar-track {
    background: transparent;
  }

  .zeros-list::-webkit-scrollbar-thumb {
    background: rgba(255, 255, 255, 0.1);
    border-radius: 2px;
  }

  .zeros-list::-webkit-scrollbar-thumb:hover {
    background: var(--accent-primary);
  }

  .zero-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    background: rgba(255, 255, 255, 0.05);
    padding: 6px 10px;
    border-radius: 6px;
  }

  .zero-val {
    font-family: monospace;
    font-size: 0.8rem;
    color: #e6edf3;
  }

  .goto-btn {
    background: transparent;
    color: var(--text-secondary);
    border: none;
    font-size: 0.7rem;
    cursor: pointer;
    padding: 2px 6px;
    border-radius: 4px;
  }

  .goto-btn:hover {
    background: rgba(255, 255, 255, 0.1);
    color: #fff;
  }

  .close-panel {
    position: absolute;
    top: 0.5rem;
    right: 0.5rem;
    background: transparent;
    border: none;
    color: var(--text-secondary);
    font-size: 1.2rem;
    cursor: pointer;
    line-height: 1;
  }

  @keyframes slideUp {
    from { transform: translateY(20px); opacity: 0; }
    to { transform: translateY(0); opacity: 1; }
  }
  :global(body) {
    margin: 0;
    padding: 0;
    overflow: hidden;
    background: #0d1117;
    color: #e6edf3;
    font-family: 'Inter', system-ui, sans-serif;
  }

  main {
    position: relative;
    width: 100vw;
    height: 100vh;
    display: flex;
    background: #0d1117;
  }

  .background-glow {
    position: absolute;
    top: 0;
    left: 0;
    width: 100%;
    height: 100%;
    background: radial-gradient(circle at 50% 50%, rgba(34, 211, 238, 0.05), transparent 70%);
    pointer-events: none;
    z-index: 0;
  }

  .full-screen {
    position: absolute;
    top: 0;
    left: 0;
    width: 100vw;
    height: 100vh;
    z-index: 1;
  }

  .view-container {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  /* Glassmorphism Sidebar */
  .sidebar {
    position: absolute;
    top: 20px;
    left: 20px;
    bottom: 20px;
    width: 320px;
    z-index: 100;
    display: flex;
    flex-direction: column;
    padding: 24px;
    border: 1px solid rgba(255, 255, 255, 0.1);
    box-shadow: 0 8px 32px 0 rgba(0, 0, 0, 0.8);
    transition: transform 0.6s cubic-bezier(0.16, 1, 0.3, 1), opacity 0.4s;
    transform: translateX(0);
    opacity: 1;
  }

  .sidebar.hidden {
    transform: translateX(-365px);
    opacity: 1;
    pointer-events: auto;
    box-shadow: none;
    border-color: transparent;
    backdrop-filter: none;
    -webkit-backdrop-filter: none;
  }

  .glass {
    background: rgba(13, 17, 23, 0.7);
    backdrop-filter: blur(20px) saturate(180%);
    -webkit-backdrop-filter: blur(20px) saturate(180%);
    border-radius: 16px;
  }

  .sidebar-toggle {
    position: absolute;
    top: 50%;
    right: -24px;
    width: 24px;
    height: 48px;
    z-index: 110;
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-left: none;
    border-radius: 0 8px 8px 0;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    color: #22d3ee;
    transition: all 0.3s;
    transform: translateY(-50%);
  }

  .sidebar-toggle:hover {
    background: rgba(34, 211, 238, 0.1);
    box-shadow: 0 0 15px rgba(34, 211, 238, 0.3);
  }

  /* Parameter Input Styling */
  .input-slider-row {
    display: flex;
    flex-direction: column;
    gap: 8px;
    margin-top: 6px;
  }

  .num-input {
    background: rgba(0, 0, 0, 0.4);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 8px;
    color: #fff;
    padding: 8px 12px;
    font-size: 0.9rem;
    font-family: 'JetBrains Mono', monospace;
    width: 100%;
    outline: none;
    transition: all 0.2s;
  }

  .num-input:focus {
    border-color: #22d3ee;
    background: rgba(34, 211, 238, 0.05);
    box-shadow: 0 0 10px rgba(34, 211, 238, 0.2);
  }

  header h1 {
    font-size: 1.5rem;
    margin: 0;
    font-weight: 800;
    letter-spacing: -0.5px;
  }

  .gradient-text {
    background: linear-gradient(135deg, #22d3ee, #8b5cf6);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
  }

  .subtitle {
    font-size: 0.7rem;
    opacity: 0.5;
    margin: 4px 0 24px 0;
    text-transform: uppercase;
    letter-spacing: 1px;
  }

  .scroll-area {
    flex: 1;
    overflow-y: auto;
    padding-right: 10px;
  }

  .scroll-area::-webkit-scrollbar {
    width: 4px;
  }

  .scroll-area::-webkit-scrollbar-thumb {
    background: rgba(255, 255, 255, 0.1);
    border-radius: 10px;
  }

  .nav-group {
    margin-bottom: 24px;
  }

  .nav-label {
    font-size: 0.65rem;
    font-weight: 700;
    color: #94a3b8;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    margin-bottom: 12px;
  }

  .nav-row {
    display: flex;
    gap: 8px;
    align-items: center;
  }

  .icon-btn {
    flex: 1;
    height: 36px;
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 8px;
    color: #64748b;
    cursor: pointer;
    transition: all 0.2s;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .icon-btn.active {
    background: rgba(34, 211, 238, 0.1);
    border-color: #22d3ee;
    color: #22d3ee;
  }

  .mode-switcher-compact {
    display: flex;
    background: rgba(0, 0, 0, 0.2);
    padding: 3px;
    border-radius: 10px;
    gap: 2px;
  }

  .mode-btn {
    flex: 1;
    padding: 6px;
    font-size: 0.7rem;
    border: none;
    background: transparent;
    color: #64748b;
    border-radius: 7px;
    cursor: pointer;
    transition: all 0.2s;
  }

  .mode-btn.active {
    background: rgba(255, 255, 255, 0.05);
    color: #fff;
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.2);
  }

  .control-row {
    margin-bottom: 12px;
  }

  .control-header {
    display: flex;
    flex-direction: column;
    gap: 8px;
    font-size: 0.75rem;
    color: #94a3b8;
  }

  .control-header span {
    display: flex;
    justify-content: space-between;
    color: #fff;
    font-family: monospace;
  }

  input[type="range"] {
    width: 100%;
    accent-color: #22d3ee;
    background: rgba(255, 255, 255, 0.05);
    height: 4px;
    border-radius: 2px;
    outline: none;
  }

  .value-display {
    margin-top: 16px;
    padding: 12px;
    background: rgba(34, 211, 238, 0.05);
    border-radius: 12px;
    border: 1px solid rgba(34, 211, 238, 0.1);
  }

  .value-display .label {
    display: block;
    font-size: 0.6rem;
    color: #22d3ee;
    margin-bottom: 4px;
    font-weight: 600;
  }

  .value-display .value {
    font-family: monospace;
    font-size: 0.85rem;
    color: #fbbf24;
  }

  .section-divider {
    height: 1px;
    background: linear-gradient(to right, transparent, rgba(255, 255, 255, 0.05), transparent);
    margin: 16px 0;
  }

  .action-pill {
    flex: 1;
    padding: 8px;
    font-size: 0.7rem;
    font-weight: 600;
    border-radius: 8px;
    border: 1px solid rgba(255, 255, 255, 0.1);
    background: rgba(255, 255, 255, 0.03);
    color: #94a3b8;
    cursor: pointer;
    transition: all 0.2s;
  }

  .action-pill.active {
    background: rgba(244, 63, 94, 0.1);
    border-color: #f43f5e;
    color: #f43f5e;
  }

  .toggle-label {
    font-size: 0.6rem;
    font-weight: 700;
    display: flex;
    align-items: center;
    gap: 4px;
    color: #64748b;
    cursor: pointer;
  }

  .range-grid {
    display: grid;
    grid-template-columns: 1fr 1fr 100px;
    gap: 12px;
    margin-top: 16px;
    align-items: flex-end;
  }

  .input-wrap span {
    display: block;
    font-size: 0.65rem;
    color: #94a3b8;
    margin-bottom: 6px;
    font-weight: 600;
  }

  .input-wrap input {
    width: 100%;
    background: rgba(0, 0, 0, 0.4);
    border: 1px solid rgba(255, 255, 255, 0.1);
    color: #fff;
    padding: 8px;
    border-radius: 8px;
    font-size: 0.8rem;
    font-family: monospace;
    outline: none;
    transition: all 0.2s;
  }

  .input-wrap input:focus {
    border-color: #22d3ee;
    background: rgba(34, 211, 238, 0.05);
  }

  .compute-btn {
    height: 32px;
    background: #fbbf24;
    color: #000;
    font-weight: 800;
    border: none;
    border-radius: 6px;
    cursor: pointer;
    font-size: 0.7rem;
    transition: all 0.2s;
  }

  .compute-btn:hover {
    transform: translateY(-1px);
    box-shadow: 0 4px 12px rgba(251, 191, 36, 0.3);
  }

  .crossings-scroll {
    max-height: 120px;
    overflow-y: auto;
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .crossing-row {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 8px 12px;
    background: rgba(255, 255, 255, 0.02);
    border: 1px solid rgba(255, 255, 255, 0.05);
    border-radius: 8px;
    color: #e6edf3;
    font-size: 0.7rem;
    cursor: pointer;
    transition: all 0.2s;
  }

  .crossing-row:hover {
    background: rgba(34, 211, 238, 0.05);
    border-color: #22d3ee;
  }

  .crossing-row.is-zero {
    border-left: 3px solid #fbbf24;
  }

  .presets-row {
    display: flex;
    flex-wrap: wrap;
    gap: 6px;
  }

  .preset-tag {
    padding: 4px 8px;
    background: rgba(255, 255, 255, 0.03);
    border: 1px solid rgba(255, 255, 255, 0.1);
    border-radius: 6px;
    color: #94a3b8;
    font-size: 0.6rem;
    cursor: pointer;
    transition: all 0.2s;
  }

  .preset-tag:hover {
    background: rgba(255, 255, 255, 0.08);
    color: #fff;
  }

</style>
