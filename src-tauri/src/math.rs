use serde::{Serialize, Deserialize};
use num_complex::Complex64;
use rayon::prelude::*;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ZetaMode {
    #[serde(rename = "zeta")]
    Zeta,
    #[serde(rename = "euler")]
    Euler,
    #[serde(rename = "reciprocal")]
    Reciprocal,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CrossingEvent {
    pub r#type: String,
    pub n: f64,
    pub re: f64,
    pub im: f64,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CalculationResult {
    pub buffer: Vec<f64>,
    pub events: Vec<CrossingEvent>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AnalyticalCrossing {
    pub t: f64,
    pub im: f64,
    pub is_zero: bool,
}

pub fn get_primes(n: usize) -> Vec<usize> {
    if n == 0 { return Vec::new(); }
    let mut primes = Vec::with_capacity(n);
    // Approximate upper bound for the n-th prime using p_n < n(ln n + ln ln n)
    let limit = if n < 6 { 15 } else {
        let fn_n = n as f64;
        (fn_n * (fn_n.ln() + fn_n.ln().ln())).ceil() as usize
    };
    let mut is_prime = vec![true; limit + 1];
    for p in 2..limit {
        if is_prime[p] {
            primes.push(p);
            if primes.len() == n { break; }
            let mut i = p * p;
            while i <= limit {
                is_prime[i] = false;
                i += p;
            }
        }
    }
    primes
}

pub fn get_moebius(n: usize) -> Vec<i8> {
    let mut mu = vec![0i8; n + 1];
    mu[1] = 1;
    
    // Simple block-based parallel sieve for Moebius
    let chunk_size = 10000;
    mu.par_chunks_mut(chunk_size).enumerate().for_each(|(chunk_idx, chunk)| {
        let start = chunk_idx * chunk_size;
        for i in 0..chunk.len() {
            let val = start + i;
            if val <= 1 { continue; }
            
            // Basic Moebius logic (simplified for parallel block)
            let mut temp = val;
            let mut count = 0;
            let mut p = 2;
            let mut is_square_free = true;
            
            while p * p <= temp {
                if temp % p == 0 {
                    count += 1;
                    temp /= p;
                    if temp % p == 0 {
                        is_square_free = false;
                        break;
                    }
                }
                p += 1;
            }
            if temp > 1 { count += 1; }
            
            chunk[i] = if !is_square_free { 0 } else if count % 2 == 1 { -1 } else { 1 };
        }
    });
    mu
}

pub fn zeta_analytical(s: Complex64, n: usize) -> Complex64 {
    if (s.re - 1.0).abs() < 1e-9 && s.im.abs() < 1e-9 {
        return Complex64::new(f64::INFINITY, 0.0);
    }
    let mut eta_sum = Complex64::new(0.0, 0.0);
    for k in 1..=n {
        let term = Complex64::new(k as f64, 0.0).powc(-s);
        if k % 2 == 1 {
            eta_sum += term;
        } else {
            eta_sum -= term;
        }
    }
    let factor = Complex64::new(1.0, 0.0) - Complex64::new(2.0, 0.0).powc(Complex64::new(1.0, 0.0) - s);
    eta_sum / factor
}

#[tauri::command]
pub fn calculate_partial_sums(
    sigma: f64,
    t: f64,
    n: usize,
    mode: ZetaMode
) -> CalculationResult {
    let mut buffer = Vec::with_capacity(n * 2);
    let mut events = Vec::new();
    
    let mut current_re: f64 = if matches!(mode, ZetaMode::Euler) { 1.0 } else { 0.0 };
    let mut current_im: f64 = 0.0;
    let mut prev_sum_re: f64 = current_re;
    let mut prev_sum_im: f64 = current_im;
    let mut sum_crossings_found = 0;

    match mode {
        ZetaMode::Zeta => {
            // Use exclusive range and add 1 for better Rayon support
            let terms: Vec<(f64, f64)> = (0..n).into_par_iter()
                .with_min_len(100)
                .map(|i| {
                    let k = (i + 1) as f64;
                    let ln_k = k.ln();
                    let arg_r = -sigma * ln_k;
                    let arg_i = -t * ln_k;
                    let exp_r = arg_r.exp();
                    (exp_r * arg_i.cos(), exp_r * arg_i.sin())
                }).collect();

            for (k_idx, (term_re, term_im)) in terms.into_iter().enumerate() {
                let k = k_idx + 1;
                current_re += term_re;
                current_im += term_im;
                
                let dist_sq = (current_re - prev_sum_re).powi(2) + (current_im - prev_sum_im).powi(2);
                if dist_sq > 1e-8 || k == 1 || k == n {
                    buffer.push(current_re);
                    buffer.push(current_im);
                    buffer.push(k as f64);
                }

                if sum_crossings_found < 5 && k > 1 {
                    if (prev_sum_re > 0.0 && current_re <= 0.0) || (prev_sum_re < 0.0 && current_re >= 0.0) {
                        let dt = if (current_re - prev_sum_re).abs() < 1e-12 { 0.0 } else { -prev_sum_re / (current_re - prev_sum_re) };
                        events.push(CrossingEvent {
                            r#type: "sum".to_string(),
                            n: (k - 1) as f64 + dt,
                            re: 0.0,
                            im: prev_sum_im + dt * (current_im - prev_sum_im),
                        });
                        sum_crossings_found += 1;
                    }
                }
                prev_sum_re = current_re;
                prev_sum_im = current_im;
            }
        },
        ZetaMode::Reciprocal => {
            let mu = get_moebius(n);
            let terms: Vec<(f64, f64)> = (0..n).into_par_iter()
                .with_min_len(100)
                .map(|i| {
                    let k = i + 1;
                    let m = mu[k] as f64;
                    let ln_k = (k as f64).ln();
                    let arg_r = -sigma * ln_k;
                    let arg_i = -t * ln_k;
                    let exp_r = arg_r.exp() * m;
                    (exp_r * arg_i.cos(), exp_r * arg_i.sin())
                }).collect();

            for (k_idx, (term_re, term_im)) in terms.into_iter().enumerate() {
                let k = k_idx + 1;
                current_re += term_re;
                current_im += term_im;
                
                let dist_sq = (current_re - prev_sum_re).powi(2) + (current_im - prev_sum_im).powi(2);
                if dist_sq > 1e-8 || k == 1 || k == n {
                    buffer.push(current_re);
                    buffer.push(current_im);
                    buffer.push(k as f64);
                }

                if sum_crossings_found < 5 && k > 1 {
                    if (prev_sum_re > 0.0 && current_re <= 0.0) || (prev_sum_re < 0.0 && current_re >= 0.0) {
                        let dt = if (current_re - prev_sum_re).abs() < 1e-12 { 0.0 } else { -prev_sum_re / (current_re - prev_sum_re) };
                        events.push(CrossingEvent {
                            r#type: "sum".to_string(),
                            n: (k - 1) as f64 + dt,
                            re: 0.0,
                            im: prev_sum_im + dt * (current_im - prev_sum_im),
                        });
                        sum_crossings_found += 1;
                    }
                }
                prev_sum_re = current_re;
                prev_sum_im = current_im;
            }
        },
        ZetaMode::Euler => {
            let primes = get_primes(n);
            let terms: Vec<Option<(f64, f64)>> = primes.into_par_iter()
                .with_min_len(50)
                .map(|p_val| {
                let p = p_val as f64;
                let ln_p = p.ln();
                let arg_r = -sigma * ln_p;
                let arg_i = -t * ln_p;
                let p_exp_r = arg_r.exp();
                let p_inv_re = p_exp_r * arg_i.cos();
                let p_inv_im = p_exp_r * arg_i.sin();
                
                let den_re = 1.0 - p_inv_re;
                let den_im = -p_inv_im;
                let mag_sq = den_re * den_re + den_im * den_im;
                
                if mag_sq == 0.0 { return None; }
                
                Some((den_re / mag_sq, -den_im / mag_sq))
            }).collect();

            for (k_idx, term_opt) in terms.into_iter().enumerate() {
                let k = k_idx + 1;
                if let Some((term_re, term_im)) = term_opt {
                    let next_re = current_re * term_re - current_im * term_im;
                    let next_im = current_re * term_im + current_im * term_re;
                    
                    current_re = next_re;
                    current_im = next_im;
                }
                
                let dist_sq = (current_re - prev_sum_re).powi(2) + (current_im - prev_sum_im).powi(2);
                if dist_sq > 1e-8 || k == 1 || k == n {
                    buffer.push(current_re);
                    buffer.push(current_im);
                    buffer.push(k as f64);
                }

                if sum_crossings_found < 5 && k > 1 {
                    if (prev_sum_re > 0.0 && current_re <= 0.0) || (prev_sum_re < 0.0 && current_re >= 0.0) {
                        let dt = if (current_re - prev_sum_re).abs() < 1e-12 { 0.0 } else { -prev_sum_re / (current_re - prev_sum_re) };
                        events.push(CrossingEvent {
                            r#type: "sum".to_string(),
                            n: (k - 1) as f64 + dt,
                            re: 0.0,
                            im: prev_sum_im + dt * (current_im - prev_sum_im),
                        });
                        sum_crossings_found += 1;
                    }
                }
                prev_sum_re = current_re;
                prev_sum_im = current_im;
            }
        }
    }

    CalculationResult { buffer, events }
}

#[tauri::command]
pub fn find_analytical_crossings(sigma: f64, t_center: f64, range: f64, steps: usize) -> Vec<AnalyticalCrossing> {
    let start = t_center - range;
    let end = t_center + range;
    let dt = (end - start) / steps as f64;

    // Parallel search across all steps
    let results: Vec<Option<AnalyticalCrossing>> = (0..steps).into_par_iter()
        .with_min_len(5)
        .map(|i| {
        let t1 = start + i as f64 * dt;
        let t2 = start + (i + 1) as f64 * dt;
        let z1 = zeta_analytical(Complex64::new(sigma, t1), 100);
        let z2 = zeta_analytical(Complex64::new(sigma, t2), 100);

        if z1.re * z2.re < 0.0 {
            let t_crossing = t1 + (t2 - t1) * (z1.re.abs() / (z1.re.abs() + z2.re.abs()));
            let z_fine = zeta_analytical(Complex64::new(sigma, t_crossing), 100);
            Some(AnalyticalCrossing {
                t: t_crossing,
                im: z_fine.im,
                is_zero: z_fine.im.abs() < 0.1,
            })
        } else {
            None
        }
    }).collect();

    results.into_iter().flatten().collect()
}

#[tauri::command]
pub fn get_zeta_value(sigma: f64, t: f64, n: usize) -> (f64, f64) {
    let z = zeta_analytical(Complex64::new(sigma, t), n);
    (z.re, z.im)
}

#[tauri::command]
pub fn calculate_zeta_path(sigma: f64, t_min: f64, t_max: f64, steps: usize) -> Vec<AnalyticalPoint> {
    let t_step = (t_max - t_min) / (steps as f64).max(1.0);
    
    (0..steps).into_par_iter()
        .with_min_len(100)
        .map(|i| {
            let t = t_min + (i as f64) * t_step;
            let val = zeta_analytical(Complex64::new(sigma, t), 500);
            AnalyticalPoint {
                re: val.re,
                im: val.im,
                t
            }
        }).collect()
}

#[derive(Serialize, Deserialize, Debug)]
pub struct AnalyticalPoint {
    pub re: f64,
    pub im: f64,
    pub t: f64,
}
