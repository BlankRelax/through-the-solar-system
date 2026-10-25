//! Numbers and durations as the readouts show them.

use crate::astro;

pub fn fmt_num(x: f64, decimals: usize) -> String {
    let s = format!("{x:.decimals$}");
    let (int, frac) = s.split_once('.').map_or((s.as_str(), None), |(a, b)| (a, Some(b)));
    let (sign, digits) = int.strip_prefix('-').map_or(("", int), |d| ("-", d));
    let mut grouped = String::new();
    for (k, ch) in digits.chars().enumerate() {
        if k > 0 && (digits.len() - k) % 3 == 0 {
            grouped.push(',');
        }
        grouped.push(ch);
    }
    match frac {
        Some(f) => format!("{sign}{grouped}.{f}"),
        None => format!("{sign}{grouped}"),
    }
}

pub fn fmt_au(au: f64) -> String {
    if au < 0.01 {
        format!("{} km", fmt_num(au * astro::AU_KM, 0))
    } else if au < 10.0 {
        format!("{} AU", fmt_num(au, 2))
    } else if au < 100.0 {
        format!("{} AU", fmt_num(au, 1))
    } else {
        format!("{} AU", fmt_num(au, 0))
    }
}

/// A light-travel time, in the largest sensible units.
pub fn fmt_duration(seconds: f64) -> String {
    let s = seconds.max(0.0);
    if s < 60.0 {
        format!("{s:.1} s")
    } else if s < 3_600.0 {
        format!("{} min {} s", (s / 60.0) as u64, (s % 60.0) as u64)
    } else if s < 86_400.0 * 2.0 {
        format!("{} h {} min", (s / 3_600.0) as u64, ((s % 3_600.0) / 60.0) as u64)
    } else if s < 86_400.0 * 365.25 {
        format!("{} days", fmt_num(s / 86_400.0, 0))
    } else {
        format!("{} years", fmt_num(s / (86_400.0 * 365.25), 1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats() {
        assert_eq!(fmt_num(1234567.891, 1), "1,234,567.9");
        assert_eq!(fmt_num(-1234.0, 0), "-1,234");
        assert_eq!(fmt_duration(499.0), "8 min 19 s");
        assert_eq!(fmt_duration(86_000.0), "23 h 53 min");
        assert_eq!(fmt_au(5.2034), "5.20 AU");
        assert_eq!(fmt_au(170.6), "171 AU");
    }
}
