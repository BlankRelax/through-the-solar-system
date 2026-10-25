//! Geometry built once at start-up.

use crate::astro;
use crate::belts::Rng;
use glam::DVec3;
use std::f64::consts::{PI, TAU};

/// A unit sphere: interleaved position (3) and texture coordinate (2), with
/// the seam duplicated so textures wrap cleanly. Longitude runs east from
/// the prime meridian (+x) toward −z; u = 0.5 at longitude 0, v = 0 at the
/// north pole (+y).
pub fn sphere(segments: u32, rings: u32) -> (Vec<f32>, Vec<u16>) {
    let mut verts = Vec::with_capacity(((segments + 1) * (rings + 1) * 5) as usize);
    for j in 0..=rings {
        let v = j as f64 / rings as f64;
        let lat = PI / 2.0 - v * PI;
        for i in 0..=segments {
            let u = i as f64 / segments as f64;
            let lon = (u - 0.5) * TAU;
            let p = [lat.cos() * lon.cos(), lat.sin(), -lat.cos() * lon.sin()];
            verts.extend([p[0] as f32, p[1] as f32, p[2] as f32, u as f32, v as f32]);
        }
    }
    let mut idx = Vec::with_capacity((segments * rings * 6) as usize);
    let row = segments + 1;
    for j in 0..rings {
        for i in 0..segments {
            let a = (j * row + i) as u16;
            let b = a + row as u16;
            idx.extend([a, b, a + 1, a + 1, b, b + 1]);
        }
    }
    (verts, idx)
}

/// A flat ring as a triangle strip: (cos θ, sin θ, edge) with edge 0 at the
/// inner radius and 1 at the outer; the shader scales it.
pub fn annulus(segments: u32) -> Vec<f32> {
    let mut out = Vec::with_capacity(((segments + 1) * 6) as usize);
    for i in 0..=segments {
        let t = TAU * i as f64 / segments as f64;
        let (s, c) = t.sin_cos();
        out.extend([c as f32, s as f32, 0.0, c as f32, s as f32, 1.0]);
    }
    out
}

/// The background stars: directions (render frame), brightness and colour.
/// Most lie near the plane of the Milky Way. Floats per star: x, y, z,
/// brightness, r, g, b.
pub fn stars(count: usize) -> Vec<f32> {
    let mut rng = Rng::new(42);
    let mut out = Vec::with_capacity(count * 7);
    for k in 0..count {
        let gal = if k % 5 < 2 {
            // Concentrated toward the galactic plane.
            let l = rng.f() * TAU;
            let b = (rng.normal() * 9.0).to_radians().clamp(-PI / 2.0, PI / 2.0);
            DVec3::new(b.cos() * l.cos(), b.cos() * l.sin(), b.sin())
        } else {
            let z: f64 = rng.range(-1.0, 1.0);
            let l = rng.f() * TAU;
            let rr = (1.0 - z * z).sqrt();
            DVec3::new(rr * l.cos(), rr * l.sin(), z)
        };
        let dir = astro::ecl_to_render(astro::equatorial_to_ecliptic(astro::galactic_to_equatorial(gal)));
        // Many faint stars, few bright ones.
        let bright = 0.18 + 0.82 * rng.f().powi(6);
        let t = rng.f();
        let rgb = if t < 0.12 {
            [0.68, 0.78, 1.0]
        } else if t < 0.55 {
            [0.95, 0.96, 1.0]
        } else if t < 0.85 {
            [1.0, 0.93, 0.8]
        } else {
            [1.0, 0.78, 0.6]
        };
        out.extend([dir.x as f32, dir.y as f32, dir.z as f32, bright as f32, rgb[0], rgb[1], rgb[2]]);
    }
    out
}

/// The heliosphere's boundaries as a sphere mesh stretched into a comet
/// shape: `nose_au` toward the interstellar wind, much longer down the tail.
/// Positions are already in display space; floats per vertex: position,
/// smoothed normal.
pub fn heliosphere(nose_au: f64, tail_stretch: f64, segments: u32, rings: u32) -> (Vec<f32>, Vec<u16>) {
    let (unit, idx) = sphere(segments, rings);
    let nose = astro::ecl_to_render(astro::ecliptic_dir(255.4, 5.2));
    let mut pos = Vec::with_capacity(unit.len() / 5);
    for v in unit.chunks(5) {
        // Point the mesh's +y at the nose.
        let d = DVec3::new(v[0] as f64, v[1] as f64, v[2] as f64);
        let q = glam::DQuat::from_rotation_arc(DVec3::Y, nose);
        let dir = q * d;
        let cos_theta = dir.dot(nose);
        let r_au = nose_au * (1.0 + tail_stretch * (1.0 - cos_theta).powf(1.5));
        pos.push(astro::to_display(astro::render_to_ecl(dir) * r_au));
    }
    // Area-weighted vertex normals; the seam's duplicate vertices are merged
    // by position so no crease shows.
    let mut normals = vec![DVec3::ZERO; pos.len()];
    for t in idx.chunks(3) {
        let (a, b, c) = (t[0] as usize, t[1] as usize, t[2] as usize);
        let n = (pos[b] - pos[a]).cross(pos[c] - pos[a]);
        for k in [a, b, c] {
            normals[k] += n;
        }
    }
    let row = segments as usize + 1;
    for j in 0..=rings as usize {
        let (first, last) = (j * row, j * row + segments as usize);
        let sum = normals[first] + normals[last];
        normals[first] = sum;
        normals[last] = sum;
    }
    let mut out = Vec::with_capacity(pos.len() * 6);
    for (p, n) in pos.iter().zip(&normals) {
        let n = n.normalize_or(DVec3::Y);
        out.extend([p.x, p.y, p.z, n.x, n.y, n.z].map(|x| x as f32));
    }
    (out, idx)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sphere_puts_greenwich_at_u_half() {
        let (v, idx) = sphere(8, 4);
        // The equator row (j = 2), column 4 is u = 0.5: longitude 0, +x.
        let k = (2 * 9 + 4) * 5;
        assert!((v[k] - 1.0).abs() < 1e-6 && v[k + 3] == 0.5);
        // Longitude 90°E (u = 0.75) is at −z.
        let k = (2 * 9 + 6) * 5;
        assert!((v[k + 2] + 1.0).abs() < 1e-6);
        assert!(idx.iter().all(|&i| (i as usize) < v.len() / 5));
    }
}
