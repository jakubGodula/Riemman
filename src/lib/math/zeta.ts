import { Complex } from './complex';
import { invoke } from '@tauri-apps/api/core';

export function zeta(s: Complex, n: number = 20): Complex {
  if (Math.abs(s.r - 1) < 1e-9 && Math.abs(s.i) < 1e-9) {
    return new Complex(Infinity, 0);
  }
  let etaSum = new Complex(0, 0);
  for (let k = 1; k <= n; k++) {
    const term = new Complex(k, 0).pow(s.negate());
    if (k % 2 === 1) etaSum = etaSum.add(term);
    else etaSum = etaSum.sub(term);
  }
  const factor = new Complex(1, 0).sub(new Complex(2, 0).pow(new Complex(1, 0).sub(s)));
  return etaSum.div(factor);
}

export interface CrossingEvent {
  type: 'sum' | 'term';
  n: number;
  re: number;
  im: number;
  totalRe?: number;
  totalIm?: number;
}

export type ZetaMode = 'zeta' | 'euler' | 'reciprocal';

export async function calculatePartialSumsWorker(
  sigma: number, 
  t: number, 
  N: number, 
  mode: ZetaMode = 'zeta'
): Promise<{ buffer: Float64Array, events: CrossingEvent[] }> {
  try {
    const result: { buffer: number[], events: any[] } = await invoke('calculate_partial_sums', {
      sigma,
      t,
      n: N,
      mode
    });
    
    return {
      buffer: new Float64Array(result.buffer),
      events: result.events.map(e => ({
        type: e.type,
        n: e.n,
        re: e.re,
        im: e.im
      }))
    };
  } catch (err) {
    console.error('Tauri invoke error:', err);
    return { buffer: new Float64Array(0), events: [] };
  }
}

export async function findAnalyticalCrossings(sigma: number, tCenter: number, range: number, steps: number = 2000): Promise<any[]> {
  try {
    const result: any[] = await invoke('find_analytical_crossings', {
      sigma,
      tCenter,
      range,
      steps
    });
    return result;
  } catch (err) {
    console.error('Tauri invoke error:', err);
    return [];
  }
}

export async function calculateZetaPath(sigma: number, tMin: number, tMax: number, steps: number = 2000): Promise<{re: number, im: number, t: number}[]> {
  try {
    const result: any[] = await invoke('calculate_zeta_path', {
      sigma,
      tMin,
      tMax,
      steps
    });
    return result;
  } catch (err) {
    console.error('Tauri invoke error:', err);
    return [];
  }
}
