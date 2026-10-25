//! The swarms drawn as points: the main asteroid belt (with its Kirkwood
//! gaps), Jupiter's Trojans, the Kuiper belt, the scattered disc and the
//! Oort cloud. Each particle is a circular orbit the vertex shader moves.

use std::f64::consts::{PI, TAU};

/// Floats per particle: a [AU], inclination, node, phase at J2000 [rad],
/// mean motion [rad/day], size, r, g, b.
pub const STRIDE: usize = 9;

/// Deterministic generator (SplitMix64), so the sky is the same every visit.
pub struct Rng(u64);

impl Rng {
    pub fn new(seed: u64) -> Rng {
        Rng(seed)
    }

    pub fn next_u64(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    /// Uniform in [0, 1).
    pub fn f(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    pub fn range(&mut self, lo: f64, hi: f64) -> f64 {
        lo + (hi - lo) * self.f()
    }

    /// Standard normal (Box–Muller).
    pub fn normal(&mut self) -> f64 {
        let u = self.f().max(1e-12);
        (-2.0 * u.ln()).sqrt() * (TAU * self.f()).cos()
    }
}

fn mean_motion(a: f64) -> f64 {
    TAU / (a.powf(1.5) * 365.25)
}

struct Push<'a>(&'a mut Vec<f32>);

impl Push<'_> {
    #[allow(clippy::too_many_arguments)]
    fn add(&mut self, a: f64, inc: f64, node: f64, phase: f64, n: f64, size: f64, rgb: [f64; 3]) {
        self.0.extend([a, inc, node, phase, n, size, rgb[0], rgb[1], rgb[2]].map(|x| x as f32));
    }
}

/// Resonances with Jupiter that clear the main belt: (a [AU], half-width).
const KIRKWOOD_GAPS: [(f64, f64); 4] = [(2.502, 0.03), (2.825, 0.025), (2.958, 0.018), (3.279, 0.04)];

/// Jupiter's mean longitude at J2000 and its mean motion (JPL elements).
const JUPITER_L0_DEG: f64 = 34.396_440_51;
const JUPITER_N_DEG_PER_CENTURY: f64 = 3_034.746_127_75;

pub fn inner_belts() -> Vec<f32> {
    let mut rng = Rng::new(7);
    let mut out = Vec::new();
    let mut p = Push(&mut out);
    // Main belt: denser in the middle, empty in the Kirkwood gaps.
    let mut placed = 0;
    while placed < 30_000 {
        let a = rng.range(2.06, 3.35);
        let density = 1.0 - ((a - 2.7) / 0.75).powi(2) * 0.6;
        if rng.f() > density || KIRKWOOD_GAPS.iter().any(|&(c, w)| (a - c).abs() < w) {
            continue;
        }
        let inc = (rng.normal() * 8.0).abs().to_radians();
        let grey = rng.range(0.55, 0.9);
        let tint = [grey, grey * 0.84, grey * 0.64];
        p.add(a, inc, rng.f() * TAU, rng.f() * TAU, mean_motion(a), rng.range(0.6, 1.4), tint);
        placed += 1;
    }
    // Trojans: clouds 60° ahead of and behind Jupiter, moving with it.
    let n = (JUPITER_N_DEG_PER_CENTURY / 36_525.0).to_radians();
    for k in 0..3_000 {
        let lead = if k % 2 == 0 { 60.0 } else { -60.0 };
        let phase = (JUPITER_L0_DEG + lead + rng.normal() * 14.0).to_radians();
        let a = 5.203 + rng.normal() * 0.07;
        let inc = (rng.normal() * 12.0).abs().to_radians();
        p.add(a, inc, rng.f() * TAU, phase, n, rng.range(0.7, 1.3), [0.75, 0.62, 0.5]);
    }
    out
}

pub fn outer_belts() -> Vec<f32> {
    let mut rng = Rng::new(11);
    let mut out = Vec::new();
    let mut p = Push(&mut out);
    let icy = |rng: &mut Rng| {
        let t = rng.range(0.0, 1.0);
        [0.55 + 0.25 * t, 0.6 + 0.15 * t, 0.7 + 0.1 * (1.0 - t)]
    };
    // Cold classical belt: thin and flat, 42–47.5 AU.
    for _ in 0..7_000 {
        let a = rng.range(42.0, 47.5);
        let inc = (rng.normal() * 2.0).abs().to_radians();
        let c = icy(&mut rng);
        p.add(a, inc, rng.f() * TAU, rng.f() * TAU, mean_motion(a), rng.range(0.8, 1.4), c);
    }
    // Hot classical belt: puffed up.
    for _ in 0..3_500 {
        let a = rng.range(40.0, 48.0);
        let inc = (rng.normal() * 13.0).abs().to_radians();
        let c = icy(&mut rng);
        p.add(a, inc, rng.f() * TAU, rng.f() * TAU, mean_motion(a), rng.range(0.8, 1.4), c);
    }
    // Plutinos: the 3:2 resonance with Neptune, at 39.4 AU.
    for _ in 0..2_500 {
        let a = 39.4 + rng.normal() * 0.15;
        let inc = (rng.normal() * 10.0).abs().to_radians();
        p.add(a, inc, rng.f() * TAU, rng.f() * TAU, mean_motion(a), rng.range(0.8, 1.4), [0.85, 0.72, 0.62]);
    }
    // Scattered disc: far out and steeply tilted.
    for _ in 0..2_500 {
        let a = (rng.range(50f64.ln(), 250f64.ln())).exp();
        let inc = (rng.normal() * 20.0).abs().to_radians();
        let c = icy(&mut rng);
        p.add(a, inc, rng.f() * TAU, rng.f() * TAU, mean_motion(a), rng.range(1.0, 1.6), c);
    }
    out
}

/// A spherical shell, 2,000 to 100,000 AU. Orbits there take millions of
/// years, so the points do not move.
pub fn oort_cloud() -> Vec<f32> {
    let mut rng = Rng::new(23);
    let mut out = Vec::new();
    let mut p = Push(&mut out);
    for _ in 0..9_000 {
        let r = rng.range(2_000f64.ln(), 100_000f64.ln()).exp();
        let z: f64 = rng.range(-1.0, 1.0);
        let lon = rng.f() * TAU;
        // A polar orbit through the point: node at its longitude, phase at
        // its latitude.
        let c = 0.7 + 0.3 * rng.f();
        p.add(r, PI / 2.0, lon, z.asin(), 0.0, rng.range(1.2, 2.2), [0.7 * c, 0.8 * c, c]);
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn belts_have_gaps_and_whole_particles() {
        let inner = inner_belts();
        assert_eq!(inner.len() % STRIDE, 0);
        let in_gap = inner
            .chunks(STRIDE)
            .filter(|p| p[0] < 4.0)
            .filter(|p| KIRKWOOD_GAPS.iter().any(|&(c, w)| ((p[0] as f64) - c).abs() < w * 0.9))
            .count();
        assert_eq!(in_gap, 0);
        assert_eq!(outer_belts().len() % STRIDE, 0);
        assert_eq!(oort_cloud().len() % STRIDE, 0);
    }
}
