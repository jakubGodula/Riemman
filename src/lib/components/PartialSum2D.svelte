<script lang="ts">
  import { onMount } from 'svelte';
  import { Complex } from '../math/complex';
  import { calculatePartialSumsWorker } from '../math/zeta';

  let { s = new Complex(0.5, 14.13), N = 500, mode = 'zeta', infiniteSum = null, trajectory = [] } = $props();
  let canvas: HTMLCanvasElement;
  let width = 600;
  let height = 600;
  let calculating = $state(false);

  let currentBuffer: Float64Array | null = null;

  async function updateBuffer() {
    calculating = true;
    const { buffer } = await calculatePartialSumsWorker(s.r, s.i, N, mode);
    currentBuffer = buffer;
    calculating = false;
    render();
  }

  function render() {
    if (!canvas || !currentBuffer) return;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    ctx.clearRect(0, 0, width, height);
    
    const count = currentBuffer.length / 3;
    if (count === 0) return;

    // Auto-scale
    const margin = 60;
    let minR = Infinity, maxR = -Infinity, minI = Infinity, maxI = -Infinity;
    
    // Bounds for partial sum
    for (let i = 0; i < count; i++) {
       const r = currentBuffer[i*3];
       const im = currentBuffer[i*3 + 1];
       minR = Math.min(minR, r); maxR = Math.max(maxR, r);
       minI = Math.min(minI, im); maxI = Math.max(maxI, im);
    }
    
    // Include infinite sum and trajectory in bounds
    if (infiniteSum) {
      minR = Math.min(minR, infiniteSum.r); maxR = Math.max(maxR, infiniteSum.r);
      minI = Math.min(minI, infiniteSum.i); maxI = Math.max(maxI, infiniteSum.i);
    }
    for (const p of trajectory) {
      minR = Math.min(minR, p.re); maxR = Math.max(maxR, p.re);
      minI = Math.min(minI, p.im); maxI = Math.max(maxI, p.im);
    }

    const scale = Math.min((width - 2 * margin) / (maxR - minR || 1), (height - 2 * margin) / (maxI - minI || 1));
    const offsetX = width / 2 - ((maxR + minR) / 2) * scale;
    const offsetY = height / 2 + ((maxI + minI) / 2) * scale;

    const getX = (re: number) => offsetX + re * scale;
    const getY = (im: number) => offsetY - im * scale;

    // Draw axes
    ctx.strokeStyle = 'rgba(255,255,255,0.1)';
    ctx.lineWidth = 1;
    ctx.beginPath();
    ctx.moveTo(getX(minR), getY(0)); ctx.lineTo(getX(maxR), getY(0));
    ctx.moveTo(getX(0), getY(minI)); ctx.lineTo(getX(0), getY(maxI));
    ctx.stroke();

    // Draw trajectory path
    if (trajectory.length > 1) {
      ctx.beginPath();
      ctx.strokeStyle = 'rgba(251, 191, 36, 0.4)';
      ctx.lineWidth = 2;
      ctx.setLineDash([5, 5]);
      ctx.moveTo(getX(trajectory[0].re), getY(trajectory[0].im));
      for (let i = 1; i < trajectory.length; i++) {
        ctx.lineTo(getX(trajectory[i].re), getY(trajectory[i].im));
      }
      ctx.stroke();
      ctx.setLineDash([]);
    }

    // Draw spiral path
    ctx.beginPath();
    ctx.strokeStyle = '#6366f1';
    ctx.lineWidth = 1.5;
    ctx.lineJoin = 'round';
    ctx.moveTo(getX(currentBuffer![0]), getY(currentBuffer![1]));
    for (let i = 1; i < count; i++) {
      ctx.lineTo(getX(currentBuffer![i * 3]), getY(currentBuffer![i * 3 + 1]));
    }
    ctx.stroke();

    // Start/End Points
    ctx.fillStyle = '#ec4899';
    ctx.beginPath(); ctx.arc(getX(currentBuffer![0]), getY(currentBuffer![1]), 3, 0, Math.PI * 2); ctx.fill();

    // N-term dot
    ctx.fillStyle = '#fff';
    ctx.beginPath(); ctx.arc(getX(currentBuffer![(count - 1) * 3]), getY(currentBuffer![(count - 1) * 3 + 1]), 4, 0, Math.PI * 2); ctx.fill();

    // Infinite Sum Dot (Golden)
    if (infiniteSum) {
      ctx.shadowBlur = 15; ctx.shadowColor = '#fbbf24';
      ctx.fillStyle = '#fbbf24';
      ctx.beginPath(); ctx.arc(getX(infiniteSum.r), getY(infiniteSum.i), 6, 0, Math.PI * 2); ctx.fill();
      ctx.strokeStyle = '#fff'; ctx.lineWidth = 2; ctx.stroke();
      ctx.shadowBlur = 0;
    }
  }

  $effect(() => {
    updateBuffer();
  });
</script>

<div class="container glass">
  <canvas bind:this={canvas} {width} {height}></canvas>
  <div class="overlay">
    <p class="title">{mode === 'zeta' ? 'Partial Sum (2D)' : mode === 'euler' ? 'Euler Product (2D)' : 'Reciprocal Sum (2D)'}</p>
    <div class="legend">
      <span>s = {s.r.toFixed(3)} + {s.i.toFixed(4)}i</span>
      <span>N = {N.toLocaleString()} terms</span>
      {#if calculating}
        <span class="pulse">Calculating...</span>
      {/if}
    </div>
  </div>
</div>

<style>
  .container {
    position: relative;
    padding: 1rem;
    display: flex;
    flex-direction: column;
    align-items: center;
    max-width: 100%;
    max-height: 100%;
    overflow: hidden;
  }
  canvas {
    border-radius: 8px;
    background: #000;
  }
  .overlay {
    position: absolute;
    top: 2rem;
    left: 2rem;
    pointer-events: none;
    text-shadow: 0 2px 4px rgba(0,0,0,0.5);
    font-family: var(--font-display);
  }
  .title {
    font-weight: 600;
    font-size: 1.1rem;
    margin: 0;
  }
  .legend {
    display: flex;
    flex-direction: column;
    font-size: 0.75rem;
    color: var(--text-secondary);
  }
  .pulse {
    color: var(--accent-primary);
    animation: pulse 1.5s infinite;
  }
  @keyframes pulse {
    0% { opacity: 0.4; }
    50% { opacity: 1; }
    100% { opacity: 0.4; }
  }
</style>
