//! Where every body is, and which way it faces, at a given moment.

use crate::astro::{self, Frame, Kepler};
use crate::catalog::{Body, Motion, OrbitSpec, Spin, BODIES};
use glam::DVec3;
use std::f64::consts::TAU;

/// Dates outside this range are clamped: the planetary elements are fitted
/// to 1800–2050, and the spacecraft paths are rough even inside it.
pub const FIRST_DAY: f64 = -73_048.5; // 1800 Jan 1
pub const LAST_DAY: f64 = 18_627.5; // 2050 Dec 31

#[derive(Clone, Copy, Debug)]
pub struct State {
    /// Where it is drawn (display units, render frame).
    pub pos: DVec3,
    /// Where it really is: heliocentric ecliptic position in AU.
    pub au: DVec3,
    pub frame: Frame,
    /// Drawn radius (display units); 0 for regions.
    pub radius: f64,
    /// Normal of the orbit plane, for moons (render frame).
    pub orbit_normal: DVec3,
    /// False before a spacecraft's launch.
    pub exists: bool,
}

pub struct Sim {
    pub parent: Vec<Option<usize>>,
    pub kepler: Vec<Option<Kepler>>,
    pub states: Vec<State>,
    pub days: f64,
}

pub fn kepler_for(spec: &OrbitSpec) -> Kepler {
    match *spec {
        OrbitSpec::Jpl(at_j2000, per_century) => Kepler { at_j2000, per_century },
        OrbitSpec::Classical { a, e, i, node, peri, m, epoch } => Kepler::classical(a, e, i, node, peri, m, epoch),
    }
}

pub fn display_radius(b: &Body) -> f64 {
    match b.motion {
        Motion::Sun => astro::SUN_DISPLAY_RADIUS,
        Motion::Region { .. } => 0.0,
        // Spacecraft are a few metres across; drawn as tiny models.
        Motion::Probe { .. } => 0.035,
        _ => astro::display_radius(b.radius_km),
    }
}

/// The node of a planet's equator on the ecliptic: a fixed reference
/// direction in its equatorial plane for laying out moon orbits.
fn equator_axes(pole: DVec3) -> (DVec3, DVec3) {
    let mut e1 = DVec3::Y.cross(pole);
    if e1.length_squared() < 1e-10 {
        e1 = DVec3::X;
    }
    let e1 = e1.normalize();
    (e1, pole.cross(e1))
}

impl Sim {
    pub fn new(days: f64) -> Sim {
        let parent = BODIES.iter().map(|b| b.parent.and_then(crate::catalog::index_of)).collect();
        let kepler = BODIES
            .iter()
            .map(|b| match &b.motion {
                Motion::Orbit(spec) => Some(kepler_for(spec)),
                _ => None,
            })
            .collect();
        let blank = State {
            pos: DVec3::ZERO,
            au: DVec3::ZERO,
            frame: Frame::IDENTITY,
            radius: 0.0,
            orbit_normal: DVec3::Y,
            exists: true,
        };
        let mut sim = Sim { parent, kepler, states: vec![blank; BODIES.len()], days };
        sim.update(days);
        sim
    }

    pub fn update(&mut self, days: f64) {
        let days = days.clamp(FIRST_DAY, LAST_DAY);
        self.days = days;
        for i in 0..BODIES.len() {
            self.states[i] = self.compute(i, days);
        }
    }

    fn compute(&self, i: usize, days: f64) -> State {
        let b = &BODIES[i];
        let radius = display_radius(b);
        let mut orbit_normal = DVec3::Y;
        let mut exists = true;
        let parent = self.parent[i].map(|p| (p, self.states[p], &BODIES[p]));

        let (pos, au) = match b.motion {
            Motion::Sun => (DVec3::ZERO, DVec3::ZERO),
            Motion::Orbit(_) => {
                let au = self.kepler[i].unwrap().position(days);
                (astro::to_display(au), au)
            }
            Motion::Fixed { lon, lat, au } => {
                let au = astro::ecliptic_dir(lon, lat) * au;
                (astro::to_display(au), au)
            }
            Motion::Region { at: (lon, lat, au), .. } => {
                let au = astro::ecliptic_dir(lon, lat) * au;
                (astro::to_display(au), au)
            }
            Motion::Probe { lon, lat, au_2025, au_per_year, launched } => {
                let year = astro::year_from_days(days);
                exists = year >= launched;
                let r = (au_2025 + (year - 2025.0) * au_per_year).max(1.0);
                let au = astro::ecliptic_dir(lon, lat) * r;
                (astro::to_display(au), au)
            }
            Motion::Lunar => {
                let (_, ps, pb) = parent.unwrap();
                let l = (218.316_5 + 13.176_396_48 * days).to_radians();
                let node = (125.044_5 - 0.052_953_8 * days).to_radians();
                let incl = 5.145_f64.to_radians();
                let u = l - node;
                let dir_ecl = DVec3::new(
                    node.cos() * u.cos() - node.sin() * u.sin() * incl.cos(),
                    node.sin() * u.cos() + node.cos() * u.sin() * incl.cos(),
                    u.sin() * incl.sin(),
                );
                let n_ecl = DVec3::new(incl.sin() * node.sin(), -incl.sin() * node.cos(), incl.cos());
                orbit_normal = astro::ecl_to_render(n_ecl);
                let dir = astro::ecl_to_render(dir_ecl);
                let d = astro::moon_display_distance(ps.radius, 384_400.0, pb.radius_km);
                (ps.pos + dir * d, ps.au + dir_ecl * (384_400.0 / astro::AU_KM))
            }
            Motion::Moon { a_km, period_days, phase, incl } => {
                let (_, ps, pb) = parent.unwrap();
                let (e1, e2) = equator_axes(ps.frame.y);
                let (si, ci) = incl.to_radians().sin_cos();
                let v2 = e2 * ci + ps.frame.y * si;
                orbit_normal = e1.cross(v2);
                let theta = phase.to_radians() + TAU * days / period_days;
                let dir = e1 * theta.cos() + v2 * theta.sin();
                let d = astro::moon_display_distance(ps.radius, a_km, pb.radius_km);
                (ps.pos + dir * d, ps.au + astro::render_to_ecl(dir) * (a_km / astro::AU_KM))
            }
        };

        let frame = match b.spin {
            Spin::Iau { ra, dec, w0, rate } => astro::iau_frame(ra, dec, w0, rate, days),
            Spin::Simple { hours } => astro::simple_frame(hours, days),
            Spin::Locked => {
                let toward_parent = parent.map(|(_, ps, _)| ps.pos - pos).unwrap_or(DVec3::X);
                Frame::from_pole_and_prime(orbit_normal, toward_parent)
            }
            Spin::FaceSun => {
                let to_sun = (-pos).normalize_or(DVec3::Y);
                Frame::from_pole_and_prime(to_sun, to_sun.any_orthonormal_vector())
            }
            Spin::None => Frame::IDENTITY,
        };

        State { pos, au, frame, radius, orbit_normal, exists }
    }

    /// Where the camera should end up to look at body `i`: an offset from
    /// the body (or, for regions, from the Sun).
    pub fn vantage(&self, i: usize) -> (usize, DVec3) {
        let b = &BODIES[i];
        let s = &self.states[i];
        if let Motion::Region { view: (lon, lat, au), .. } = b.motion {
            return (0, astro::to_display(astro::ecliptic_dir(lon, lat) * au));
        }
        // Look at the lit side, from a little above and to one side.
        let to_sun = (-s.pos).normalize_or(DVec3::X);
        let side = to_sun.cross(DVec3::Y).normalize_or(DVec3::Z);
        let dir = (to_sun * 0.75 + side * 0.55 + DVec3::Y * 0.3).normalize();
        (i, dir * self.view_distance(i))
    }

    pub fn view_distance(&self, i: usize) -> f64 {
        let b = &BODIES[i];
        let r = self.states[i].radius;
        match b.motion {
            Motion::Sun => r * 5.0,
            Motion::Probe { .. } => r * 9.0,
            _ if b.look.rings.is_some() => r * 5.5,
            _ => (r * 4.0).max(0.12),
        }
    }

    /// Heliocentric distance (AU) of a point in display space.
    pub fn au_from_sun(p: DVec3) -> f64 {
        astro::display_to_au(p.length())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::catalog::index_of;

    #[test]
    fn distances_are_plausible_in_2025() {
        let sim = Sim::new(astro::days_from_year(2025.0));
        let r = |id| sim.states[index_of(id).unwrap()].au.length();
        assert!((r("earth") - 0.983).abs() < 0.01, "earth {}", r("earth"));
        assert!((r("eris") - 95.6).abs() < 1.5, "eris {}", r("eris"));
        assert!((r("sedna") - 83.5).abs() < 2.0, "sedna {}", r("sedna"));
        assert!((r("halley") - 35.0).abs() < 1.5, "halley {}", r("halley"));
        assert!((r("makemake") - 52.6).abs() < 1.0, "makemake {}", r("makemake"));
        assert!((r("haumea") - 50.0).abs() < 1.5, "haumea {}", r("haumea"));
        let jupiter = r("jupiter");
        assert!((4.9..5.5).contains(&jupiter));
    }

    #[test]
    fn moons_stay_outside_their_planets_and_rings() {
        let sim = Sim::new(9000.0);
        for (i, b) in BODIES.iter().enumerate() {
            if let Some(p) = sim.parent[i] {
                if matches!(b.motion, Motion::Moon { .. } | Motion::Lunar) {
                    let gap = (sim.states[i].pos - sim.states[p].pos).length();
                    let mut clear = sim.states[p].radius + sim.states[i].radius;
                    if let Some(rings) = BODIES[p].look.rings {
                        clear = clear.max(sim.states[p].radius * rings.outer as f64);
                    }
                    assert!(gap > clear, "{} overlaps its parent", b.id);
                }
            }
        }
    }

    #[test]
    fn moon_is_full_when_opposite_the_sun() {
        // Full moon on 2025-03-14 06:55 UTC (total lunar eclipse).
        let days = astro::days_from_unix_ms(1_741_935_300_000.0);
        let sim = Sim::new(days);
        let earth = sim.states[index_of("earth").unwrap()].au;
        let moon = sim.states[index_of("moon").unwrap()].au;
        let elongation = (moon - earth).normalize().dot(earth.normalize());
        assert!(elongation > 0.97, "{elongation}");
    }

    #[test]
    fn locked_moons_face_their_planet() {
        let sim = Sim::new(1234.5);
        let i = index_of("moon").unwrap();
        let p = sim.parent[i].unwrap();
        let to_earth = (sim.states[p].pos - sim.states[i].pos).normalize();
        assert!(sim.states[i].frame.x.dot(to_earth) > 0.99);
    }
}
