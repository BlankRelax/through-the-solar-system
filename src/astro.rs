//! Time, orbits, rotation and reference frames, and the compressed scale the
//! game draws in.
//!
//! Positions are heliocentric, in astronomical units, in the J2000 ecliptic
//! frame (x toward the March equinox, z toward ecliptic north). The renderer
//! works in "display units" in a y-up frame; `to_display` converts.

use glam::DVec3;
use std::f64::consts::{PI, TAU};

pub const AU_KM: f64 = 149_597_870.7;
pub const EARTH_RADIUS_KM: f64 = 6_371.0;
pub const LIGHT_SECONDS_PER_AU: f64 = 499.004_784;
pub const DAYS_PER_YEAR: f64 = 365.25;
const DAYS_PER_CENTURY: f64 = 36_525.0;
const UNIX_EPOCH_JD: f64 = 2_440_587.5;
const J2000_JD: f64 = 2_451_545.0;
/// Obliquity of the ecliptic at J2000 (IAU 2006).
const OBLIQUITY_DEG: f64 = 23.439_279;

/// Display units per sqrt(AU): 1 AU is 60 units out, 100 AU is 600.
pub const DISTANCE_SCALE: f64 = 60.0;
/// The Sun is drawn smaller than the rule below would make it, so Mercury
/// still has room to orbit.
pub const SUN_DISPLAY_RADIUS: f64 = 8.0;

// --- time ---------------------------------------------------------------------

/// Days since J2000.0 (2000 January 1, 12:00 TT) for a Unix time in milliseconds.
/// TT and UTC differ by about a minute, which nothing here can show.
pub fn days_from_unix_ms(ms: f64) -> f64 {
    ms / 86_400_000.0 + UNIX_EPOCH_JD - J2000_JD
}

pub fn unix_ms_from_days(days: f64) -> f64 {
    (days - UNIX_EPOCH_JD + J2000_JD) * 86_400_000.0
}

/// Days since J2000 for a decimal (Julian) year such as 2025.0.
pub fn days_from_year(year: f64) -> f64 {
    (year - 2000.0) * DAYS_PER_YEAR
}

pub fn year_from_days(days: f64) -> f64 {
    2000.0 + days / DAYS_PER_YEAR
}

/// Calendar date and time (UTC, proleptic Gregorian) for days since J2000.
pub fn civil_from_days(days: f64) -> (i64, u32, u32, u32, u32) {
    let ms = unix_ms_from_days(days);
    let total_minutes = (ms / 60_000.0).floor() as i64;
    let day = total_minutes.div_euclid(1440);
    let minute_of_day = total_minutes.rem_euclid(1440);
    // Howard Hinnant's days-to-civil algorithm.
    let z = day + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    let y = yoe + era * 400 + i64::from(m <= 2);
    (y, m, d, (minute_of_day / 60) as u32, (minute_of_day % 60) as u32)
}

// --- frames -------------------------------------------------------------------

/// Ecliptic (z north) to the renderer's y-up frame. A proper rotation, so
/// cross products keep their meaning.
pub fn ecl_to_render(v: DVec3) -> DVec3 {
    DVec3::new(v.x, v.z, -v.y)
}

pub fn render_to_ecl(v: DVec3) -> DVec3 {
    DVec3::new(v.x, -v.z, v.y)
}

pub fn equatorial_to_ecliptic(v: DVec3) -> DVec3 {
    let (s, c) = OBLIQUITY_DEG.to_radians().sin_cos();
    DVec3::new(v.x, v.y * c + v.z * s, -v.y * s + v.z * c)
}

pub fn ecliptic_to_equatorial(v: DVec3) -> DVec3 {
    let (s, c) = OBLIQUITY_DEG.to_radians().sin_cos();
    DVec3::new(v.x, v.y * c - v.z * s, v.y * s + v.z * c)
}

/// Unit vector in the equatorial frame for a right ascension and declination.
pub fn radec(ra_deg: f64, dec_deg: f64) -> DVec3 {
    let (sa, ca) = ra_deg.to_radians().sin_cos();
    let (sd, cd) = dec_deg.to_radians().sin_cos();
    DVec3::new(cd * ca, cd * sa, sd)
}

/// Unit vector in the ecliptic frame for an ecliptic longitude and latitude.
pub fn ecliptic_dir(lon_deg: f64, lat_deg: f64) -> DVec3 {
    let (sl, cl) = lon_deg.to_radians().sin_cos();
    let (sb, cb) = lat_deg.to_radians().sin_cos();
    DVec3::new(cb * cl, cb * sl, sb)
}

/// Equatorial (J2000) to galactic coordinates (the standard IAU rotation).
pub fn equatorial_to_galactic(v: DVec3) -> DVec3 {
    DVec3::new(
        -0.054_875_560_4 * v.x - 0.873_437_090_2 * v.y - 0.483_835_015_5 * v.z,
        0.494_109_427_9 * v.x - 0.444_829_630_0 * v.y + 0.746_982_244_5 * v.z,
        -0.867_666_149_0 * v.x - 0.198_076_373_4 * v.y + 0.455_983_776_2 * v.z,
    )
}

/// Galactic to equatorial: the transpose of the rotation above.
pub fn galactic_to_equatorial(v: DVec3) -> DVec3 {
    DVec3::new(
        -0.054_875_560_4 * v.x + 0.494_109_427_9 * v.y - 0.867_666_149_0 * v.z,
        -0.873_437_090_2 * v.x - 0.444_829_630_0 * v.y - 0.198_076_373_4 * v.z,
        -0.483_835_015_5 * v.x + 0.746_982_244_5 * v.y + 0.455_983_776_2 * v.z,
    )
}

// --- the compressed scale ------------------------------------------------------

/// Distance from the Sun on screen for a real distance: the square root keeps
/// Mercury and the Oort cloud in one flyable scene.
pub fn au_to_display(r_au: f64) -> f64 {
    DISTANCE_SCALE * r_au.max(0.0).sqrt()
}

pub fn display_to_au(r: f64) -> f64 {
    let x = r / DISTANCE_SCALE;
    x * x
}

/// A heliocentric ecliptic position in AU, moved to where the game draws it:
/// same direction from the Sun, compressed distance.
pub fn to_display(p_au: DVec3) -> DVec3 {
    let r = p_au.length();
    if r < 1e-12 {
        return DVec3::ZERO;
    }
    ecl_to_render(p_au) * (au_to_display(r) / r)
}

/// The inverse of `to_display`.
pub fn from_display(p: DVec3) -> DVec3 {
    let r = p.length();
    if r < 1e-12 {
        return DVec3::ZERO;
    }
    render_to_ecl(p) * (display_to_au(r) / r)
}

/// Drawn radius for a real radius. Sizes grow as R^0.55, so Jupiter is still
/// clearly the giant but a 10 km moon is not invisible.
pub fn display_radius(radius_km: f64) -> f64 {
    (0.9 * (radius_km / EARTH_RADIUS_KM).powf(0.55)).max(0.012)
}

/// Drawn distance of a moon from its planet's centre, in display units.
pub fn moon_display_distance(parent_display: f64, a_km: f64, parent_km: f64) -> f64 {
    parent_display * (1.0 + 1.2 * (a_km / parent_km - 1.0).max(0.1).sqrt())
}

// --- orbits ---------------------------------------------------------------------

/// Keplerian elements in the form of JPL's "Approximate Positions of the
/// Planets" (Standish): a [AU], e, I, L, long. of perihelion, long. of the
/// ascending node [deg], each with a rate per Julian century.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Kepler {
    pub at_j2000: [f64; 6],
    pub per_century: [f64; 6],
}

impl Kepler {
    /// From classical elements valid at `epoch_year`: semi-major axis [AU],
    /// eccentricity, inclination, node, argument of perihelion and mean
    /// anomaly [deg]. The mean motion follows from Kepler's third law.
    pub fn classical(a: f64, e: f64, i: f64, node: f64, peri: f64, mean_anomaly: f64, epoch_year: f64) -> Self {
        let lp = node + peri;
        let n_per_century = 360.0 / a.powf(1.5) * 100.0;
        let centuries = (epoch_year - 2000.0) / 100.0;
        let l0 = mean_anomaly + lp - n_per_century * centuries;
        Kepler {
            at_j2000: [a, e, i, l0, lp, node],
            per_century: [0.0, 0.0, 0.0, n_per_century, 0.0, 0.0],
        }
    }

    fn elements(&self, days: f64) -> [f64; 6] {
        let t = days / DAYS_PER_CENTURY;
        let mut el = self.at_j2000;
        for (x, r) in el.iter_mut().zip(self.per_century) {
            *x += r * t;
        }
        el
    }

    /// Heliocentric ecliptic position in AU.
    pub fn position(&self, days: f64) -> DVec3 {
        let [a, e, i, l, lp, node] = self.elements(days);
        let m = wrap_pi((l - lp).to_radians());
        point_on_orbit(a, e, i, node, lp - node, solve_kepler(m, e))
    }

    /// `n` points around the whole orbit as it is at `days`, evenly spaced in
    /// eccentric anomaly (denser near perihelion, where the orbit bends most).
    pub fn path(&self, days: f64, n: usize) -> Vec<DVec3> {
        let [a, e, i, _, lp, node] = self.elements(days);
        (0..n)
            .map(|k| point_on_orbit(a, e, i, node, lp - node, TAU * k as f64 / n as f64))
            .collect()
    }

    pub fn semi_major_axis(&self) -> f64 {
        self.at_j2000[0]
    }

    pub fn period_days(&self) -> f64 {
        self.at_j2000[0].powf(1.5) * DAYS_PER_YEAR
    }
}

fn wrap_pi(x: f64) -> f64 {
    (x + PI).rem_euclid(TAU) - PI
}

/// Eccentric anomaly for a mean anomaly (radians), by Newton's method.
pub fn solve_kepler(m: f64, e: f64) -> f64 {
    let mut ea = if e < 0.8 { m } else { PI.copysign(m) };
    for _ in 0..60 {
        let step = (ea - e * ea.sin() - m) / (1.0 - e * ea.cos());
        ea -= step;
        if step.abs() < 1e-13 {
            break;
        }
    }
    ea
}

fn point_on_orbit(a: f64, e: f64, i_deg: f64, node_deg: f64, peri_deg: f64, ea: f64) -> DVec3 {
    let xp = a * (ea.cos() - e);
    let yp = a * (1.0 - e * e).sqrt() * ea.sin();
    let (so, co) = peri_deg.to_radians().sin_cos();
    let (sn, cn) = node_deg.to_radians().sin_cos();
    let (si, ci) = i_deg.to_radians().sin_cos();
    DVec3::new(
        (co * cn - so * sn * ci) * xp + (-so * cn - co * sn * ci) * yp,
        (co * sn + so * cn * ci) * xp + (-so * sn + co * cn * ci) * yp,
        so * si * xp + co * si * yp,
    )
}

// --- rotation -------------------------------------------------------------------

/// A body-fixed frame in render coordinates: `x` points at the prime meridian
/// on the equator, `y` at the north pole, `z = x × y`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Frame {
    pub x: DVec3,
    pub y: DVec3,
    pub z: DVec3,
}

impl Frame {
    pub const IDENTITY: Frame = Frame { x: DVec3::X, y: DVec3::Y, z: DVec3::Z };

    pub fn from_pole_and_prime(pole: DVec3, prime: DVec3) -> Frame {
        let y = pole.normalize();
        let x = (prime - y * prime.dot(y)).normalize();
        Frame { x, y, z: x.cross(y) }
    }
}

/// IAU rotation model: north pole at (ra, dec), prime meridian at
/// W = w0 + rate · days measured from the node of the body's equator on the
/// Earth's equator.
pub fn iau_frame(ra_deg: f64, dec_deg: f64, w0: f64, rate_deg_per_day: f64, days: f64) -> Frame {
    let pole = radec(ra_deg, dec_deg);
    let (sa, ca) = ra_deg.to_radians().sin_cos();
    let node = DVec3::new(-sa, ca, 0.0);
    let (sw, cw) = (w0 + rate_deg_per_day * days).to_radians().sin_cos();
    let prime = node * cw + pole.cross(node) * sw;
    let to_render = |v| ecl_to_render(equatorial_to_ecliptic(v));
    Frame::from_pole_and_prime(to_render(pole), to_render(prime))
}

/// A steady spin about ecliptic north (for bodies whose pole is unknown).
pub fn simple_frame(period_hours: f64, days: f64) -> Frame {
    let angle = TAU * days * 24.0 / period_hours;
    let (s, c) = angle.sin_cos();
    let prime = ecl_to_render(DVec3::new(c, s, 0.0));
    Frame::from_pole_and_prime(DVec3::Y, prime)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn earth() -> Kepler {
        Kepler {
            at_j2000: [1.000_002_61, 0.016_711_23, -0.000_015_31, 100.464_571_66, 102.937_681_93, 0.0],
            per_century: [0.000_005_62, -0.000_043_92, -0.012_946_68, 35_999.372_449_81, 0.323_273_64, 0.0],
        }
    }

    #[test]
    fn earth_at_j2000() {
        // Earth–Moon barycentre on 2000 Jan 1.5: about (-0.177, 0.967, 0) AU.
        let p = earth().position(0.0);
        assert!((p.x + 0.177).abs() < 0.01 && (p.y - 0.967).abs() < 0.01, "{p:?}");
    }

    #[test]
    fn perihelion_in_early_january() {
        let k = earth();
        let jan3 = days_from_year(2025.0) + 2.5;
        let jul4 = jan3 + 182.0;
        assert!(k.position(jan3).length() < 0.984);
        assert!(k.position(jul4).length() > 1.015);
    }

    #[test]
    fn kepler_solver_handles_comets() {
        for &e in &[0.0, 0.2, 0.9, 0.967, 0.99] {
            for k in -20..=20 {
                let m = k as f64 * 0.157;
                let ea = solve_kepler(m, e);
                assert!((ea - e * ea.sin() - m).abs() < 1e-9, "e={e} m={m}");
            }
        }
    }

    #[test]
    fn display_scale_round_trips() {
        let p = DVec3::new(3.0, -4.0, 1.0);
        assert!((from_display(to_display(p)) - p).length() < 1e-9);
        assert!((au_to_display(1.0) - DISTANCE_SCALE).abs() < 1e-12);
    }

    #[test]
    fn calendar() {
        assert_eq!(civil_from_days(0.0), (2000, 1, 1, 12, 0));
        let ms = 1_761_350_400_000.0; // 2025-10-25T00:00:00Z
        assert_eq!(civil_from_days(days_from_unix_ms(ms)), (2025, 10, 25, 0, 0));
    }

    #[test]
    fn galactic_rotation_is_orthonormal() {
        let v = DVec3::new(0.3, -0.5, 0.81).normalize();
        assert!((galactic_to_equatorial(equatorial_to_galactic(v)) - v).length() < 1e-8);
        // The galactic centre lies at RA 266.4°, Dec −28.9°.
        let gc = galactic_to_equatorial(DVec3::X);
        let ra = gc.y.atan2(gc.x).to_degrees().rem_euclid(360.0);
        let dec = gc.z.asin().to_degrees();
        assert!((ra - 266.4).abs() < 0.2 && (dec + 28.94).abs() < 0.2, "{ra} {dec}");
    }

    #[test]
    fn earth_rotation_puts_noon_on_greenwich_in_daylight() {
        // At 2000 Jan 1 12:00 UT the Sun is near Greenwich's meridian, so the
        // prime meridian should point roughly at the Sun (opposite Earth's
        // heliocentric position).
        let f = iau_frame(0.0, 90.0, 190.147, 360.985_623_5, 0.0);
        let sun_dir = ecl_to_render(-earth().position(0.0)).normalize();
        let along = f.x.dot(sun_dir);
        assert!(along > 0.9, "{along}");
    }
}
