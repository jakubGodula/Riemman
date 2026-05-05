<script lang="ts">
  import { T, Canvas } from '@threlte/core';
  import { OrbitControls, Grid, HTML } from '@threlte/extras';
  import { Complex } from '../math/complex';
  import { calculatePartialSumsWorker, type CrossingEvent } from '../math/zeta';
  import * as THREE from 'three';

  let { 
    s = new Complex(0.5, 14.13), 
    N = 500, 
    mode = 'zeta', 
    infiniteSum = null,
    trajectory = [] 
  } = $props();
  let calculating = $state(false);

  // Core State
  let points = $state<THREE.Vector3[]>([]);
  let crossings = $state<CrossingEvent[]>([]);
  let geometry = $state<THREE.BufferGeometry>(new THREE.BufferGeometry());
  let trajectoryGeometry = $state<THREE.BufferGeometry>(new THREE.BufferGeometry());

  // Navigation State
  let scrubN = $state(0);
  let zLength = $derived(N * 0.01);
  let cameraTarget = $derived(new THREE.Vector3(0, 0, scrubN * 0.01));
  
  type ViewAngle = 'isometric' | 'top' | 'front' | 'side';
  let viewAngle = $state<ViewAngle>('isometric');

  let cameraOffset = $derived.by(() => {
    switch (viewAngle) {
      case 'top': return new THREE.Vector3(0, 30, 0.1);
      case 'front': return new THREE.Vector3(0, 0, -30);
      case 'side': return new THREE.Vector3(30, 0, 0);
      case 'isometric': default: return new THREE.Vector3(15, 15, 15);
    }
  });

  let cameraPosition = $derived(new THREE.Vector3(cameraOffset.x, cameraOffset.y, scrubN * 0.01 + cameraOffset.z));

  async function updateBuffer() {
    if (calculating) return;
    calculating = true;
    
    try {
      const { buffer, events } = await calculatePartialSumsWorker(s.r, s.i, N, mode);
      crossings = events;
      
      const count = buffer.length / 3;
      const positions = new Float32Array(count * 3);
      const colors = new Float32Array(count * 3);
      const color = new THREE.Color();

      for (let i = 0; i < count; i++) {
        const re = buffer[i * 3];
        const im = buffer[i * 3 + 1];
        const nIdx = buffer[i * 3 + 2];
        
        positions[i * 3] = re;
        positions[i * 3 + 1] = im;
        positions[i * 3 + 2] = nIdx * 0.01;
        
        const alpha = i / count;
        color.setHSL(0.6 + alpha * 0.2, 0.8, 0.4 + alpha * 0.2);
        colors[i * 3] = color.r;
        colors[i * 3 + 1] = color.g;
        colors[i * 3 + 2] = color.b;
      }
      
      const newGeo = new THREE.BufferGeometry();
      newGeo.setAttribute('position', new THREE.BufferAttribute(positions, 3));
      newGeo.setAttribute('color', new THREE.BufferAttribute(colors, 3));
      geometry = newGeo;

      points = [
        new THREE.Vector3(positions[0], positions[1], positions[2]),
        new THREE.Vector3(positions[positions.length - 3], positions[positions.length - 2], positions[positions.length - 1])
      ];
    } catch (err) {
      console.error('3D Update Error:', err);
    } finally {
      calculating = false;
    }
  }

  function updateTrajectory() {
    if (!trajectory || trajectory.length === 0) {
      trajectoryGeometry = new THREE.BufferGeometry();
      return;
    }
    
    const trajPos = new Float32Array(trajectory.length * 3);
    for (let i = 0; i < trajectory.length; i++) {
      trajPos[i * 3] = trajectory[i].re;
      trajPos[i * 3 + 1] = trajectory[i].im;
      trajPos[i * 3 + 2] = 0.05;
    }
    const tGeo = new THREE.BufferGeometry();
    tGeo.setAttribute('position', new THREE.BufferAttribute(trajPos, 3));
    trajectoryGeometry = tGeo;
  }

  // Reactive updates
  $effect(() => {
    s; N; mode;
    import.meta.env.SSR || updateBuffer();
  });

  $effect(() => {
    trajectory;
    import.meta.env.SSR || updateTrajectory();
  });
</script>

<div class="container glass">
  <Canvas>
    <T.Color attach="background" args={['#050508']} />

    <T.PerspectiveCamera
      makeDefault
      position={[cameraPosition.x, cameraPosition.y, cameraPosition.z]}
      near={0.1}
      far={10000}
    >
      <OrbitControls 
        enableDamping 
        target={[cameraTarget.x, cameraTarget.y, cameraTarget.z]} 
      />
    </T.PerspectiveCamera>

    <T.AmbientLight intensity={1.5} />
    <T.HemisphereLight args={[0xffffff, 0x000000, 1.0]} />
    <T.PointLight position={[20, 20, 20]} intensity={5} />

    <!-- Numerical Axis Markers (Subtle Ticks & High-Contrast Labels) -->
    {#each [-20, -10, -5, -4, -3, -2, -1, 1, 2, 3, 4, 5, 10, 20] as val}
      <!-- Real Axis Ticks (Red) -->
      <T.Mesh position={[val, 0, 0]}>
        <T.BoxGeometry args={[0.02, 0.1, 0.02]} />
        <T.MeshBasicMaterial color="#f43f5e" />
        <HTML position={[0, -0.075, 0.05]} center>
          <div style="font-size: 10px; color: #fff; font-family: sans-serif; background: #f43f5e; padding: 1px 3px; border-radius: 2px; font-weight: bold; white-space: nowrap; box-shadow: 0 1px 2px rgba(0,0,0,0.5);">
            {val}
          </div>
        </HTML>
      </T.Mesh>
      
      <!-- Imaginary Axis Ticks (Blue) -->
      <T.Mesh position={[0, val, 0]}>
        <T.BoxGeometry args={[0.1, 0.02, 0.02]} />
        <T.MeshBasicMaterial color="#0ea5e9" />
        <HTML position={[-0.075, 0, 0.05]} center>
          <div style="font-size: 10px; color: #fff; font-family: sans-serif; background: #0ea5e9; padding: 1px 3px; border-radius: 2px; font-weight: bold; white-space: nowrap; box-shadow: 0 1px 2px rgba(0,0,0,0.5);">
            {val}i
          </div>
        </HTML>
      </T.Mesh>
    {/each}

    <!-- Bottom Grid (n=0 plane) -->
    <Grid
      position={[0, 0, 0]}
      gridSize={[200, 200]}
      cellColor="#333"
      sectionColor="#666"
      plane="xy"
    />

    <!-- Imaginary Axial Plane (Re=0) -->
    <Grid
      position={[0, 0, zLength / 2]}
      gridSize={[200, zLength + 40]}
      cellColor="#083344"
      sectionColor="#0ea5e9"
      plane="zy"
    />
    <T.Mesh position={[0, 0, zLength / 2]} rotation={[0, Math.PI / 2, 0]}>
      <T.PlaneGeometry args={[zLength + 40, 200]} />
      <T.MeshBasicMaterial color="#0ea5e9" transparent opacity={0.03} side={THREE.DoubleSide} depthWrite={false} />
    </T.Mesh>

    <!-- Real Axial Plane (Im=0) -->
    <Grid
      position={[0, 0, zLength / 2]}
      gridSize={[200, zLength + 40]}
      cellColor="#450a0a"
      sectionColor="#f43f5e"
      plane="xz"
    />
    <T.Mesh position={[0, 0, zLength / 2]} rotation={[Math.PI / 2, 0, 0]}>
      <T.PlaneGeometry args={[zLength + 40, 200]} />
      <T.MeshBasicMaterial color="#f43f5e" transparent opacity={0.03} side={THREE.DoubleSide} depthWrite={false} />
    </T.Mesh>

    {#if points.length > 0}
      <T.Line geometry={geometry}>
        <T.LineBasicMaterial vertexColors={true} linewidth={2} />
      </T.Line>

      {#if trajectory.length > 1}
        <T.Line geometry={trajectoryGeometry} renderOrder={10}>
          <T.LineBasicMaterial color="#fbbf24" linewidth={3} transparent opacity={0.8} depthTest={false} />
        </T.Line>
      {/if}

      <!-- Markers -->
      {#each crossings as event}
        <T.Mesh position={[event.re, event.im, event.n * 0.01]}>
          <T.SphereGeometry args={[0.009, 16, 16]} />
          <T.MeshStandardMaterial 
            color="#22d3ee" 
            emissive="#22d3ee"
            emissiveIntensity={1.0}
            metalness={0.8}
            roughness={0.2}
          />
        </T.Mesh>
      {/each}

      <!-- Starting Point Marker -->
      <T.Mesh position={[points[0].x, points[0].y, points[0].z]}>
        <T.SphereGeometry args={[0.005, 16, 16]} />
        <T.MeshBasicMaterial color="#ec4899" />
      </T.Mesh>

      <!-- Current N-Term Marker (Small White) -->
      <T.Mesh position={[points[1].x, points[1].y, points[1].z]}>
        <T.SphereGeometry args={[0.008, 16, 16]} />
        <T.MeshBasicMaterial color="#fff" />
      </T.Mesh>

      {#if infiniteSum}
        <!-- Bright Infinite Sum (Analytical Limit) -->
        <T.Group position={[infiniteSum.r, infiniteSum.i, 0.05]}>
          <T.Mesh renderOrder={11}>
            <T.SphereGeometry args={[0.02, 32, 32]} />
            <T.MeshBasicMaterial color="#fff" depthTest={false} />
          </T.Mesh>
          <T.Mesh renderOrder={11}>
            <T.SphereGeometry args={[0.05, 32, 32]} />
            <T.MeshBasicMaterial color="#fbbf24" transparent opacity={0.8} depthTest={false} />
          </T.Mesh>
          <T.Mesh renderOrder={11}>
            <T.SphereGeometry args={[0.1, 32, 32]} />
            <T.MeshBasicMaterial color="#fbbf24" transparent opacity={0.4} depthTest={false} />
          </T.Mesh>
          <T.PointLight color="#fbbf24" intensity={5} distance={10} decay={2} />
        </T.Group>
      {/if}
    {/if}
  </Canvas>

  <div class="overlay">
    <p class="title">{mode === 'zeta' ? '3D Partial Sum' : mode === 'euler' ? '3D Euler Product' : '3D Reciprocal Sum'}</p>
    <div class="legend">
      <span>N = {N.toLocaleString()} terms ({points.length} nodes)</span>
      <span>Scrubber: n = {scrubN.toLocaleString()}</span>
      <div class="markers-legend">
        <span class="dot aqua"></span> Imaginary Plane (Re=0)
        <span class="dot red"></span> Real Plane (Im=0)
      </div>
      <div class="markers-legend" style="margin-top: 2px;">
        <span class="dot aqua small"></span> Crossing (Re=0)
        <span class="dot gold small" style="margin-left: 8px;"></span> Infinite Sum ζ(s)
      </div>
      {#if calculating}
        <span class="pulse text-accent">Calculating...</span>
      {/if}
    </div>
  </div>

  <div class="view-controls">
    <button class:active={viewAngle === 'isometric'} onclick={() => viewAngle = 'isometric'}>Isometric</button>
    <button class:active={viewAngle === 'top'} onclick={() => viewAngle = 'top'}>Top</button>
    <button class:active={viewAngle === 'front'} onclick={() => viewAngle = 'front'}>Front</button>
    <button class:active={viewAngle === 'side'} onclick={() => viewAngle = 'side'}>Side</button>
  </div>

  <div class="bottom-overlay">
    <div class="scrubber-card glass">
      <div class="scrubber-header">
        <span class="label">Camera Scrubber</span>
        <span class="value">{((scrubN / N) * 100).toFixed(1)}%</span>
      </div>
      <input 
        type="range" 
        min="0" 
        max={N} 
        step="1" 
        bind:value={scrubN} 
        class="scrubber-slider" 
      />
    </div>
  </div>
</div>

<style>
  .container {
    position: relative;
    width: 100%;
    height: 100%;
    border-radius: 12px;
    overflow: hidden;
    background: #000;
  }
  .overlay {
    position: absolute;
    top: 1.5rem;
    left: 1.5rem;
    pointer-events: none;
    z-index: 10;
    font-family: var(--font-display);
  }
  .title {
    font-weight: 600;
    font-size: 1.2rem;
    color: var(--text-primary);
  }
  .legend {
    display: flex;
    flex-direction: column;
    font-size: 0.75rem;
    color: var(--text-secondary);
    gap: 4px;
  }
  .markers-legend {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-top: 4px;
  }
  .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
    display: inline-block;
  }
  .aqua { background: #06b6d4; box-shadow: 0 0 8px #06b6d4; }
  .red { background: #ef4444; box-shadow: 0 0 8px #ef4444; }
  .gold { background: #fbbf24; box-shadow: 0 0 10px #fbbf24; }
  .dot.small { width: 5px; height: 5px; margin-left: 1.5px; }
  .pulse {
    margin-top: 8px;
    color: var(--accent-primary);
    animation: pulse 1.5s infinite;
  }
  @keyframes pulse {
    0% { opacity: 0.4; }
    50% { opacity: 1; }
    100% { opacity: 0.4; }
  }

  .bottom-overlay {
    position: absolute;
    bottom: 2rem;
    left: 50%;
    transform: translateX(-50%);
    width: 100%;
    max-width: 600px;
    padding: 0 2rem;
    pointer-events: none;
    z-index: 20;
  }

  .scrubber-card {
    padding: 1rem 1.5rem;
    pointer-events: auto;
    display: flex;
    flex-direction: column;
    gap: 0.75rem;
    border: 1px solid rgba(255, 255, 255, 0.1);
  }

  .scrubber-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .scrubber-header .label {
    font-size: 0.65rem;
    text-transform: uppercase;
    letter-spacing: 1px;
    color: var(--text-secondary);
    font-weight: 600;
  }

  .scrubber-header .value {
    font-size: 0.7rem;
    color: var(--accent-primary);
    font-family: monospace;
  }

  .scrubber-slider {
    width: 100%;
    height: 4px;
    appearance: none;
    background: rgba(255, 255, 255, 0.1);
    border-radius: 2px;
    outline: none;
  }

  .scrubber-slider::-webkit-slider-thumb {
    appearance: none;
    width: 16px;
    height: 16px;
    border-radius: 50%;
    background: var(--accent-primary);
    cursor: pointer;
    border: 2px solid #fff;
    box-shadow: 0 0 10px rgba(99, 102, 241, 0.5);
    transition: transform 0.1s;
  }

  .scrubber-slider::-webkit-slider-thumb:hover {
    transform: scale(1.2);
  }

  .view-controls {
    position: absolute;
    top: 1.5rem;
    right: 1.5rem;
    display: flex;
    gap: 8px;
    background: rgba(0, 0, 0, 0.4);
    backdrop-filter: blur(8px);
    padding: 6px;
    border-radius: 8px;
    border: 1px solid rgba(255, 255, 255, 0.1);
    z-index: 20;
    pointer-events: auto;
  }

  .view-controls button {
    background: transparent;
    color: var(--text-secondary);
    border: 1px solid transparent;
    padding: 4px 10px;
    font-size: 0.7rem;
    border-radius: 4px;
    cursor: pointer;
    transition: all 0.2s;
  }

  .view-controls button:hover {
    background: rgba(255, 255, 255, 0.1);
    color: var(--text-primary);
  }

  .view-controls button.active {
    background: rgba(99, 102, 241, 0.2);
    color: var(--text-primary);
    border-color: var(--accent-primary);
  }
</style>
