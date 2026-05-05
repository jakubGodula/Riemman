<script lang="ts">
  import { onMount } from 'svelte';
  import { Complex } from '../math/complex';
  import { zeta } from '../math/zeta';

  let canvas: HTMLCanvasElement;
  let { width = 600, height = 600, rangeX = [-2, 2], rangeY = [-2, 2] } = $props();

  function complexToColor(z: Complex): [number, number, number] {
    const phase = z.arg();
    const mag = z.abs();

    // Hue from phase (0 to 360)
    const h = ((phase + Math.PI) / (2 * Math.PI)) * 360;
    
    // Saturation and Lightness
    // Simple magnitude representation: repeating lines for magnitude
    const logMag = Math.log(mag + 1);
    const l = 50 + 20 * Math.sin(logMag * 10);
    const s = 80;

    return [h, s, l];
  }

  function render() {
    if (!canvas) return;
    const ctx = canvas.getContext('2d');
    if (!ctx) return;

    const imageData = ctx.createImageData(width, height);
    const data = imageData.data;

    for (let py = 0; py < height; py++) {
      for (let px = 0; px < width; px++) {
        const x = rangeX[0] + (px / width) * (rangeX[1] - rangeX[0]);
        const y = rangeY[1] - (py / height) * (rangeY[1] - rangeY[0]);

        const s = new Complex(x, y);
        const z = zeta(s, 12); // Using Borwein with 12 terms for speed

        const [h, s_val, l] = complexToColor(z);
        const [r, g, b] = hslToRgb(h / 360, s_val / 100, l / 100);

        const idx = (py * width + px) * 4;
        data[idx] = r;
        data[idx + 1] = g;
        data[idx + 2] = b;
        data[idx + 3] = 255;
      }
    }
    ctx.putImageData(imageData, 0, 0);
  }

  function hslToRgb(h: number, s: number, l: number): [number, number, number] {
    let r, g, b;
    if (s === 0) {
      r = g = b = l;
    } else {
      const q = l < 0.5 ? l * (1 + s) : l + s - l * s;
      const p = 2 * l - q;
      r = hue2rgb(p, q, h + 1/3);
      g = hue2rgb(p, q, h);
      b = hue2rgb(p, q, h - 1/3);
    }
    return [Math.round(r * 255), Math.round(g * 255), Math.round(b * 255)];
  }

  function hue2rgb(p: number, q: number, t: number) {
    if (t < 0) t += 1;
    if (t > 1) t -= 1;
    if (t < 1/6) return p + (q - p) * 6 * t;
    if (t < 1/2) return q;
    if (t < 2/3) return p + (q - p) * (2/3 - t) * 6;
    return p;
  }

  onMount(() => {
    // We render once or on range change
    // Using a timeout to not block main thread immediately
    setTimeout(render, 100);
  });
</script>

<div class="container glass">
  <canvas bind:this={canvas} {width} {height}></canvas>
  <div class="overlay">
    <p>Domain Coloring: ζ(s)</p>
    <div class="legend">H: Phase, L: Magnitude oscillations</div>
  </div>
</div>

<style>
  .container {
    position: relative;
    padding: 1rem;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1rem;
    max-width: 100%;
    max-height: 100%;
    overflow: hidden;
  }
  canvas {
    border-radius: 8px;
    max-width: 100%;
    height: auto;
    background: #000;
  }
  .overlay {
    position: absolute;
    top: 2rem;
    left: 2rem;
    pointer-events: none;
    text-shadow: 0 2px 4px rgba(0,0,0,0.5);
  }
  .legend {
    font-size: 0.75rem;
    color: var(--text-secondary);
  }
</style>
