//! Everything the game knows about each object: how it moves, how it looks,
//! and the annotations — verified facts (each with where it comes from) and
//! hypotheses (each labelled with how settled it is).
//!
//! Bodies are listed parents-first, in tour order, from the Sun outward.

/// What kind of object a body is; shown above its name.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Star,
    Planet,
    DwarfPlanet,
    Moon,
    Asteroid,
    Comet,
    Tno,
    Spacecraft,
    Region,
    Hypothetical,
}

impl Kind {
    pub fn label(self) -> &'static str {
        match self {
            Kind::Star => "Star",
            Kind::Planet => "Planet",
            Kind::DwarfPlanet => "Dwarf planet",
            Kind::Moon => "Moon",
            Kind::Asteroid => "Asteroid",
            Kind::Comet => "Comet",
            Kind::Tno => "Trans-Neptunian object",
            Kind::Spacecraft => "Spacecraft",
            Kind::Region => "Region",
            Kind::Hypothetical => "Hypothetical planet",
        }
    }

    pub fn css(self) -> &'static str {
        match self {
            Kind::Star => "star",
            Kind::Planet => "planet",
            Kind::DwarfPlanet => "dwarf",
            Kind::Moon => "moon",
            Kind::Asteroid | Kind::Comet | Kind::Tno => "minor",
            Kind::Spacecraft => "craft",
            Kind::Region => "region",
            Kind::Hypothetical => "hypothetical",
        }
    }
}

/// How settled a hypothesis is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Status {
    /// The explanation most researchers favour, though not proven.
    Leading,
    /// Backed by direct evidence, but not yet beyond doubt.
    Supported,
    /// Serious competing explanations.
    Debated,
    /// No accepted explanation yet.
    Open,
    /// A claim that later work has challenged.
    Disputed,
    /// An idea with little or no direct evidence.
    Speculative,
}

impl Status {
    pub fn label(self) -> &'static str {
        match self {
            Status::Leading => "Leading hypothesis",
            Status::Supported => "Supported by evidence",
            Status::Debated => "Debated",
            Status::Open => "Open question",
            Status::Disputed => "Disputed",
            Status::Speculative => "Speculative",
        }
    }

    pub fn css(self) -> &'static str {
        match self {
            Status::Leading => "leading",
            Status::Supported => "supported",
            Status::Debated => "debated",
            Status::Open => "open",
            Status::Disputed => "disputed",
            Status::Speculative => "speculative",
        }
    }
}

pub struct Fact {
    pub text: &'static str,
    pub source: &'static str,
}

pub struct Idea {
    pub status: Status,
    pub text: &'static str,
}

const fn fact(text: &'static str, source: &'static str) -> Fact {
    Fact { text, source }
}

const fn idea(status: Status, text: &'static str) -> Idea {
    Idea { status, text }
}

use Status::*;

/// Orbit around the Sun.
#[derive(Clone, Copy, Debug)]
pub enum OrbitSpec {
    /// JPL "Approximate Positions of the Planets", valid 1800–2050:
    /// a, e, I, L, ϖ, Ω at J2000 and their rates per century.
    Jpl([f64; 6], [f64; 6]),
    /// Osculating elements at an epoch (decimal year), propagated as a fixed
    /// ellipse: a [AU], e, i, Ω, ω, M [deg].
    Classical { a: f64, e: f64, i: f64, node: f64, peri: f64, m: f64, epoch: f64 },
}

#[derive(Clone, Copy, Debug)]
pub enum Motion {
    Sun,
    Orbit(OrbitSpec),
    /// Circular orbit around the parent. `incl` is measured from the
    /// parent's equator; over 90° means retrograde.
    Moon { a_km: f64, period_days: f64, phase: f64, incl: f64 },
    /// Earth's Moon: mean longitude and regressing node, so its phase and
    /// position on the sky are right to within a few degrees.
    Lunar,
    /// Heading out of the Solar System in a fixed direction (ecliptic
    /// longitude/latitude), at a steady speed.
    Probe { lon: f64, lat: f64, au_2025: f64, au_per_year: f64, launched: f64 },
    /// A fixed point (ecliptic longitude/latitude, distance in AU).
    Fixed { lon: f64, lat: f64, au: f64 },
    /// A region: the label sits at `at`, and flying there takes you to the
    /// vantage point `view`, looking back at the Sun.
    Region { at: (f64, f64, f64), view: (f64, f64, f64) },
}

#[derive(Clone, Copy, Debug)]
pub enum Spin {
    /// IAU pole (RA, Dec) and prime meridian W = w0 + rate · days.
    Iau { ra: f64, dec: f64, w0: f64, rate: f64 },
    /// Tidally locked: one face toward the parent, pole along the orbit normal.
    Locked,
    /// A spacecraft keeping its dish (+y) pointed back at the Sun and Earth.
    FaceSun,
    /// Rotation period in hours about ecliptic north (pole not well known).
    Simple { hours: f64 },
    None,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Shade {
    /// Glowing surface.
    Sun,
    /// Hard surface, lit by the Sun.
    Rock,
    /// Banded gas or cloud, softer terminator.
    Gas,
    /// Day map, night lights and clouds.
    Earth,
    /// A see-through silhouette (for things that may not exist).
    Ghost,
    /// Nothing to draw (regions).
    None,
}

#[derive(Clone, Copy, Debug)]
pub struct Rings {
    /// Inner and outer edge in planet (equatorial) radii.
    pub inner: f32,
    pub outer: f32,
    pub tex: Option<&'static str>,
    pub color: [f32; 3],
    pub opacity: f32,
}

#[derive(Clone, Copy, Debug)]
pub struct Lobe {
    pub offset: [f32; 3],
    pub scale: [f32; 3],
}

/// Procedural surfaces (see `planet.frag`), for worlds with no texture map.
pub mod feature {
    pub const CRATERED: i32 = 0;
    pub const IO: i32 = 1;
    pub const EUROPA: i32 = 2;
    pub const GANYMEDE: i32 = 3;
    pub const CALLISTO: i32 = 4;
    pub const TITAN: i32 = 5;
    pub const ENCELADUS: i32 = 6;
    pub const MIMAS: i32 = 7;
    pub const TRITON: i32 = 8;
    pub const PLUTO: i32 = 9;
    pub const CHARON: i32 = 10;
    pub const CERES: i32 = 11;
    pub const ICY: i32 = 12;
    pub const MIRANDA: i32 = 13;
    /// Spacecraft: smooth painted surfaces, no terrain.
    pub const SPACECRAFT: i32 = 14;
}

#[derive(Clone, Copy, Debug)]
pub struct Look {
    pub shade: Shade,
    pub tex: Option<&'static str>,
    pub night: Option<&'static str>,
    pub clouds: Option<&'static str>,
    pub feature: i32,
    pub color: [f32; 3],
    pub color2: [f32; 3],
    /// Crater depth for procedural surfaces (0 = none).
    pub craters: f32,
    /// Semi-axes relative to the radius: x (toward the prime meridian), y
    /// (polar), z.
    pub shape: [f32; 3],
    /// Irregular, lumpy outline (small bodies).
    pub bumpy: f32,
    /// Atmosphere glow colour and strength (0 = none).
    pub atmosphere: [f32; 4],
    pub rings: Option<Rings>,
    pub lobes: &'static [Lobe],
    /// Colour of the dot drawn when the body is too small to see.
    pub dot: [f32; 3],
}

impl Look {
    const ROCK: Look = Look {
        shade: Shade::Rock,
        tex: None,
        night: None,
        clouds: None,
        feature: feature::CRATERED,
        color: [0.5, 0.5, 0.5],
        color2: [0.35, 0.34, 0.33],
        craters: 1.0,
        shape: [1.0, 1.0, 1.0],
        bumpy: 0.0,
        atmosphere: [0.0; 4],
        rings: None,
        lobes: &[],
        dot: [0.8, 0.8, 0.8],
    };
    /// A Voyager: high-gain dish, ten-sided bus, and three booms
    /// (magnetometer, nuclear power, science instruments).
    const VOYAGER: Look = Look {
        color: [0.86, 0.85, 0.82],
        color2: [0.72, 0.6, 0.32],
        feature: feature::SPACECRAFT,
        craters: 0.0,
        lobes: &[
            Lobe { offset: [0.0, 0.18, 0.0], scale: [0.62, 0.1, 0.62] },
            Lobe { offset: [0.0, -0.05, 0.0], scale: [0.26, 0.14, 0.26] },
            Lobe { offset: [0.0, -0.08, 1.35], scale: [0.025, 0.025, 1.3] },
            Lobe { offset: [-0.78, -0.08, 0.0], scale: [0.55, 0.06, 0.06] },
            Lobe { offset: [0.75, -0.06, -0.1], scale: [0.5, 0.05, 0.05] },
        ],
        dot: [0.55, 0.85, 1.0],
        ..Look::ROCK
    };
    /// New Horizons: a triangular body with the dish on top and one power boom.
    const NEW_HORIZONS: Look = Look {
        color: [0.85, 0.8, 0.6],
        color2: [0.7, 0.58, 0.3],
        feature: feature::SPACECRAFT,
        craters: 0.0,
        lobes: &[
            Lobe { offset: [0.0, 0.3, 0.0], scale: [0.5, 0.08, 0.5] },
            Lobe { offset: [0.0, -0.05, 0.0], scale: [0.42, 0.28, 0.32] },
            Lobe { offset: [0.62, -0.1, 0.0], scale: [0.32, 0.06, 0.06] },
        ],
        dot: [0.55, 0.85, 1.0],
        ..Look::ROCK
    };
    const NONE: Look = Look { shade: Shade::None, dot: [0.9, 0.8, 0.6], ..Look::ROCK };
}

pub struct Body {
    pub id: &'static str,
    pub name: &'static str,
    pub kind: Kind,
    pub parent: Option<&'static str>,
    pub tagline: &'static str,
    /// Mean (or equatorial, for the giants) radius in km; 0 for regions.
    pub radius_km: f64,
    pub motion: Motion,
    pub spin: Spin,
    pub look: Look,
    pub stats: &'static [(&'static str, &'static str)],
    pub facts: &'static [Fact],
    pub ideas: &'static [Idea],
    pub missions: &'static [&'static str],
}

impl Body {
    pub fn is_region(&self) -> bool {
        matches!(self.motion, Motion::Region { .. })
    }
}

pub fn index_of(id: &str) -> Option<usize> {
    BODIES.iter().position(|b| b.id == id)
}

// Sources, named once so they read the same everywhere.
const NASA: &str = "NASA Science";
const NASA_FACTS: &str = "NASA Planetary Fact Sheet (NSSDCA)";
const JPL: &str = "NASA/JPL";
const ESA: &str = "ESA";
const IAU: &str = "International Astronomical Union";
const MESSENGER: &str = "NASA MESSENGER mission";
const CASSINI: &str = "NASA/ESA/ASI Cassini–Huygens";
const JUNO: &str = "NASA Juno mission";
const GALILEO: &str = "NASA Galileo mission";
const NEW_HORIZONS: &str = "NASA New Horizons mission";
const VOYAGER: &str = "NASA/JPL Voyager mission";
const DAWN: &str = "NASA Dawn mission";
const MPC: &str = "IAU Minor Planet Center";

pub static BODIES: &[Body] = &[
    // ------------------------------------------------------------------ Sun
    Body {
        id: "sun",
        name: "The Sun",
        kind: Kind::Star,
        parent: None,
        tagline: "An ordinary yellow dwarf star that holds 99.86% of the Solar System's mass.",
        radius_km: 695_700.0,
        motion: Motion::Sun,
        spin: Spin::Iau { ra: 286.13, dec: 63.87, w0: 84.176, rate: 14.184_4 },
        look: Look { shade: Shade::Sun, tex: Some("sun"), color: [1.0, 0.85, 0.55], dot: [1.0, 0.95, 0.8], ..Look::ROCK },
        stats: &[
            ("Type", "G2V main-sequence star"),
            ("Radius", "695,700 km (109 Earths)"),
            ("Mass", "1.989 × 10³⁰ kg (333,000 Earths)"),
            ("Surface temperature", "about 5,500 °C (5,772 K)"),
            ("Core temperature", "about 15 million °C"),
            ("Rotation", "about 25 days at the equator, 35 near the poles"),
            ("Age", "about 4.6 billion years"),
            ("Light to Earth", "8 min 20 s"),
        ],
        facts: &[
            fact("The Sun contains about 99.86% of all the mass in the Solar System.", NASA),
            fact("Every second its core fuses roughly 600 million tonnes of hydrogen into helium; about 4 million tonnes of that mass becomes energy (E = mc²).", NASA),
            fact("Sunlight reaches Earth in about 8 minutes 20 seconds, but the energy it carries took tens of thousands of years or more to work its way out from the core.", NASA),
            fact("The Sun is not solid, so it rotates at different rates: about 25 days at the equator and about 35 days near the poles.", NASA),
            fact("Its magnetic field flips roughly every 11 years at the peak of the sunspot cycle. NASA and NOAA announced in October 2024 that the Sun had reached the maximum period of Solar Cycle 25.", "NASA / NOAA"),
            fact("The corona, its outer atmosphere, is over a million degrees — hundreds of times hotter than the visible surface below it.", NASA),
            fact("On 24 December 2024 Parker Solar Probe passed 6.1 million km from the Sun's surface at about 692,000 km/h, the closest and fastest any spacecraft has been.", "NASA / Johns Hopkins APL"),
            fact("The Sun is roughly halfway through its life as a main-sequence star; in about 5 billion years it will swell into a red giant.", NASA),
        ],
        ideas: &[
            idea(Open, "Why is the corona so much hotter than the surface? The two main candidates — energy carried up by magnetic (Alfvén) waves, and countless tiny 'nanoflares' from snapping magnetic field lines — may both contribute. Parker Solar Probe and Solar Orbiter are testing them."),
            idea(Debated, "The 'faint young Sun' paradox: 4 billion years ago the Sun was about 70% as bright as today, yet early Earth had liquid water. A stronger greenhouse effect (CO₂, methane) is the usual answer, but no explanation is agreed."),
            idea(Leading, "The Sun was born in a cluster with hundreds or thousands of siblings. Short-lived isotopes such as aluminium-26 in meteorites suggest a nearby massive star or supernova seeded the young Solar System."),
            idea(Debated, "Will the red-giant Sun swallow Earth? Mercury and Venus will be engulfed; whether Earth escapes depends on how much mass the Sun loses (widening Earth's orbit), and models disagree."),
        ],
        missions: &["SOHO (ESA/NASA, 1995–)", "Solar Dynamics Observatory (NASA, 2010–)", "Parker Solar Probe (NASA, 2018–)", "Solar Orbiter (ESA/NASA, 2020–)", "Aditya-L1 (ISRO, 2023–)"],
    },
    // -------------------------------------------------------------- Mercury
    Body {
        id: "mercury",
        name: "Mercury",
        kind: Kind::Planet,
        parent: Some("sun"),
        tagline: "The smallest planet and the closest to the Sun: a shrinking iron world.",
        radius_km: 2_439.7,
        motion: Motion::Orbit(OrbitSpec::Jpl(
            [0.387_099_27, 0.205_635_93, 7.004_979_02, 252.250_323_50, 77.457_796_28, 48.330_765_93],
            [0.000_000_37, 0.000_019_06, -0.005_947_49, 149_472.674_111_75, 0.160_476_89, -0.125_340_81],
        )),
        spin: Spin::Iau { ra: 281.010_3, dec: 61.415_5, w0: 329.598_8, rate: 6.138_510_8 },
        look: Look { tex: Some("mercury"), dot: [0.75, 0.72, 0.7], ..Look::ROCK },
        stats: &[
            ("Mean radius", "2,439.7 km (0.38 Earth)"),
            ("Mass", "0.055 Earth"),
            ("Distance from Sun", "0.39 AU (46–70 million km)"),
            ("Year", "88 Earth days"),
            ("Solar day", "176 Earth days"),
            ("Surface temperature", "−180 °C to 430 °C"),
            ("Moons", "0"),
        ],
        facts: &[
            fact("Mercury spins exactly three times for every two orbits of the Sun (a 3:2 spin–orbit resonance), so one solar day lasts two of its years.", NASA),
            fact("Its iron core spans about 85% of the planet's radius, a far larger share than Earth's.", MESSENGER),
            fact("Despite being closest to the Sun it is not the hottest planet — Venus is. With almost no atmosphere, Mercury's night side falls to about −180 °C.", NASA),
            fact("MESSENGER confirmed water ice in permanently shadowed craters near Mercury's poles in 2012.", MESSENGER),
            fact("As its interior cooled the planet contracted, shrinking its radius by up to about 7 km and wrinkling the surface into cliffs (lobate scarps) hundreds of kilometres long.", MESSENGER),
            fact("It has a global magnetic field about 1% as strong as Earth's.", MESSENGER),
            fact("The Caloris Basin, about 1,550 km across, is one of the largest impact basins in the Solar System.", NASA),
            fact("MESSENGER found surprisingly high amounts of volatile elements such as sulfur and potassium — unexpected so close to the Sun.", MESSENGER),
        ],
        ideas: &[
            idea(Debated, "Why is the core so large? Candidates: a giant impact stripped away much of the rocky mantle; the young Sun vaporised the outer layers; or iron-rich material was concentrated in the innermost part of the planet-forming disc."),
            idea(Open, "'Hollows' — bright, shallow pits found nowhere else — seem to form as some volatile material escapes from the rock, possibly sulfur compounds. BepiColombo is due to study them."),
        ],
        missions: &["Mariner 10 (NASA, flybys 1974–75)", "MESSENGER (NASA, orbited 2011–2015)", "BepiColombo (ESA/JAXA, launched 2018; due to enter orbit in late 2026)"],
    },
    // ---------------------------------------------------------------- Venus
    Body {
        id: "venus",
        name: "Venus",
        kind: Kind::Planet,
        parent: Some("sun"),
        tagline: "Earth's near-twin in size, wrapped in a crushing, scorching CO₂ atmosphere.",
        radius_km: 6_051.8,
        motion: Motion::Orbit(OrbitSpec::Jpl(
            [0.723_335_66, 0.006_776_72, 3.394_676_05, 181.979_099_50, 131.602_467_18, 76.679_842_55],
            [0.000_003_90, -0.000_041_07, -0.000_788_90, 58_517.815_387_29, 0.002_683_29, -0.277_694_18],
        )),
        spin: Spin::Iau { ra: 272.76, dec: 67.16, w0: 160.20, rate: -1.481_368_8 },
        look: Look {
            shade: Shade::Gas,
            tex: Some("venus_atmosphere"),
            atmosphere: [0.95, 0.85, 0.6, 0.9],
            dot: [1.0, 0.95, 0.8],
            ..Look::ROCK
        },
        stats: &[
            ("Mean radius", "6,051.8 km (0.95 Earth)"),
            ("Mass", "0.815 Earth"),
            ("Distance from Sun", "0.72 AU (108 million km)"),
            ("Year", "224.7 Earth days"),
            ("Rotation", "243 Earth days, backwards"),
            ("Surface temperature", "about 465 °C"),
            ("Surface pressure", "about 92 times Earth's"),
            ("Moons", "0"),
        ],
        facts: &[
            fact("Venus is the hottest planet, about 465 °C at the surface — hot enough to melt lead — because its thick CO₂ atmosphere traps heat in a runaway greenhouse effect.", NASA),
            fact("Surface pressure is about 92 times Earth's, like being roughly 900 m under the sea.", NASA),
            fact("It rotates backwards compared with most planets, and so slowly that one rotation (243 Earth days) takes longer than one orbit (225 days).", NASA_FACTS),
            fact("Its clouds are made of sulfuric acid droplets, and the upper atmosphere races around the planet in about 4 Earth days — 60 times faster than the surface turns.", ESA),
            fact("The Soviet Venera 7 made the first successful landing on another planet in 1970; Venera 13 sent back the first colour pictures from the surface in 1982 and survived for 127 minutes.", NASA),
            fact("NASA's Magellan orbiter mapped about 98% of the surface by radar between 1990 and 1994.", JPL),
            fact("After the Moon it is the brightest natural object in the night sky.", NASA),
        ],
        ideas: &[
            idea(Supported, "Venus is volcanically active today. Comparing Magellan radar images taken months apart, researchers in 2023 found a volcanic vent near Maat Mons that changed shape and grew — the strongest evidence yet of a current eruption."),
            idea(Debated, "Did Venus once have oceans? Some climate models allow liquid water for billions of years; others suggest the young planet was always too hot for water to rain out. NASA's planned DAVINCI and VERITAS missions aim to find out."),
            idea(Disputed, "In 2020 astronomers reported phosphine gas in the clouds, a possible sign of life. Later analyses disputed the detection and its interpretation, and it remains unconfirmed."),
            idea(Open, "Why does Venus spin backwards? A giant impact, or slow tidal and atmospheric forces that flipped or reversed its spin, have both been proposed."),
        ],
        missions: &["Venera 4–16 (USSR, 1967–1983)", "Magellan (NASA, 1990–1994)", "Venus Express (ESA, 2006–2014)", "Akatsuki (JAXA, 2015–2024)", "Planned: VERITAS and DAVINCI (NASA), EnVision (ESA)"],
    },
    // ---------------------------------------------------------------- Earth
    Body {
        id: "earth",
        name: "Earth",
        kind: Kind::Planet,
        parent: Some("sun"),
        tagline: "Home: the only world known to have liquid surface oceans and life.",
        radius_km: 6_371.0,
        motion: Motion::Orbit(OrbitSpec::Jpl(
            [1.000_002_61, 0.016_711_23, -0.000_015_31, 100.464_571_66, 102.937_681_93, 0.0],
            [0.000_005_62, -0.000_043_92, -0.012_946_68, 35_999.372_449_81, 0.323_273_64, 0.0],
        )),
        spin: Spin::Iau { ra: 0.0, dec: 90.0, w0: 190.147, rate: 360.985_623_5 },
        look: Look {
            shade: Shade::Earth,
            tex: Some("earth_daymap"),
            night: Some("earth_nightmap"),
            clouds: Some("earth_clouds"),
            atmosphere: [0.35, 0.6, 1.0, 1.0],
            dot: [0.55, 0.75, 1.0],
            ..Look::ROCK
        },
        stats: &[
            ("Mean radius", "6,371 km"),
            ("Mass", "5.97 × 10²⁴ kg"),
            ("Distance from Sun", "1 AU (149.6 million km)"),
            ("Year", "365.26 days"),
            ("Rotation", "23 h 56 min (sidereal)"),
            ("Axial tilt", "23.4°"),
            ("Mean surface temperature", "about 15 °C"),
            ("Moons", "1"),
        ],
        facts: &[
            fact("Earth is the only place known to host life. About 71% of its surface is covered by water.", NASA),
            fact("It is the densest planet in the Solar System, averaging 5.51 g/cm³.", NASA_FACTS),
            fact("Radiometric dating of meteorites and the oldest Earth minerals puts Earth's age at about 4.54 billion years.", "U.S. Geological Survey"),
            fact("Earth is the only planet known to have active plate tectonics, which recycles its crust over hundreds of millions of years.", NASA),
            fact("Earth is closest to the Sun in early January. Seasons come from the 23.4° tilt of its axis, not from changing distance.", NASA),
            fact("Its magnetic field, made by the churning liquid-iron outer core, has reversed many times; the last full reversal was about 780,000 years ago.", "U.S. Geological Survey"),
            fact("Tidal friction from the Moon is slowly lengthening the day, by roughly 2 milliseconds per century.", NASA),
            fact("The glowing patterns on the night side here are city lights, as seen by satellites.", "NASA Earth Observatory"),
        ],
        ideas: &[
            idea(Debated, "Where did Earth's water come from? Water-rich asteroids like carbonaceous chondrites match its chemistry well, but some studies suggest the rocky building blocks that formed Earth already held enough water."),
            idea(Open, "How did life begin? Leading ideas include deep-sea hydrothermal vents and shallow 'warm little ponds' on land; many researchers favour an early 'RNA world' in which RNA both stored information and catalysed reactions."),
            idea(Supported, "'Snowball Earth': strong geological evidence shows ice reached the tropics during glaciations about 720 and 650 million years ago. How completely the oceans froze is still debated."),
        ],
        missions: &["Thousands of Earth-observing satellites", "International Space Station, continuously crewed since November 2000"],
    },
    // ----------------------------------------------------------------- Moon
    Body {
        id: "moon",
        name: "The Moon",
        kind: Kind::Moon,
        parent: Some("earth"),
        tagline: "Earth's companion, and the only other world people have walked on.",
        radius_km: 1_737.4,
        motion: Motion::Lunar,
        spin: Spin::Locked,
        look: Look { tex: Some("moon"), dot: [0.85, 0.85, 0.82], ..Look::ROCK },
        stats: &[
            ("Mean radius", "1,737.4 km (0.27 Earth)"),
            ("Mass", "0.0123 Earth"),
            ("Distance from Earth", "384,400 km (average)"),
            ("Orbit", "27.3 days; phases repeat every 29.5 days"),
            ("Surface gravity", "0.17 g"),
            ("Surface temperature", "−173 °C to 127 °C"),
        ],
        facts: &[
            fact("The Moon is tidally locked: it turns once per orbit, so the same face always points toward Earth.", NASA),
            fact("It is drifting away from Earth by about 3.8 cm per year, measured by bouncing lasers off reflectors left by Apollo astronauts.", NASA),
            fact("Twelve astronauts walked on the Moon between 1969 and 1972 and brought back 382 kg of rock and soil.", NASA),
            fact("The dark 'seas' (maria) are plains of basalt lava that erupted roughly 3 to 3.5 billion years ago.", NASA),
            fact("China's Chang'e 4 made the first landing on the far side in January 2019, and Chang'e 6 returned the first far-side samples in June 2024.", "China National Space Administration"),
            fact("Water ice is trapped in permanently shadowed craters near the poles; NASA's LCROSS impact confirmed it in 2009.", NASA),
            fact("India's Chandrayaan-3 landed near the south pole in August 2023, the first landing that close to it.", "ISRO"),
        ],
        ideas: &[
            idea(Leading, "The giant-impact hypothesis: about 4.5 billion years ago a Mars-sized body (often called Theia) struck the young Earth, and the Moon formed from the debris. A puzzle remains — the Moon's isotopes are nearly identical to Earth's — which variants of the impact try to explain."),
            idea(Open, "Why are the near and far sides so different? The near side has most of the maria and thinner crust. Heat from the young Earth, or the aftermath of the huge South Pole–Aitken impact, are among the explanations."),
            idea(Debated, "Is the deepest part of the mantle still partly molten? Apollo seismometers and tidal measurements hint at a soft, low-viscosity layer above the core."),
        ],
        missions: &["Luna (USSR, 1959–1976)", "Apollo (NASA, crewed landings 1969–1972)", "Lunar Reconnaissance Orbiter (NASA, 2009–)", "Chang'e 3–6 (China, 2013–2024)", "Chandrayaan-3 (India, 2023)", "Artemis (NASA): Artemis II, the first crewed flight around the Moon since 1972, was scheduled for 2026"],
    },
    // ----------------------------------------------------------------- Mars
    Body {
        id: "mars",
        name: "Mars",
        kind: Kind::Planet,
        parent: Some("sun"),
        tagline: "A cold desert world that was once wet — the most explored planet after Earth.",
        radius_km: 3_389.5,
        motion: Motion::Orbit(OrbitSpec::Jpl(
            [1.523_710_34, 0.093_394_10, 1.849_691_42, -4.553_432_05, -23.943_629_59, 49.559_538_91],
            [0.000_018_47, 0.000_078_82, -0.008_131_31, 19_140.302_684_99, 0.444_410_88, -0.292_573_43],
        )),
        spin: Spin::Iau { ra: 317.681_43, dec: 52.886_5, w0: 176.630, rate: 350.891_982_26 },
        look: Look { tex: Some("mars"), atmosphere: [0.9, 0.55, 0.4, 0.35], dot: [1.0, 0.6, 0.45], ..Look::ROCK },
        stats: &[
            ("Mean radius", "3,389.5 km (0.53 Earth)"),
            ("Mass", "0.107 Earth"),
            ("Distance from Sun", "1.52 AU (228 million km)"),
            ("Year", "687 Earth days"),
            ("Solar day (sol)", "24 h 39 min"),
            ("Axial tilt", "25.2°"),
            ("Surface pressure", "about 0.6% of Earth's"),
            ("Moons", "2 (Phobos and Deimos)"),
        ],
        facts: &[
            fact("Olympus Mons is the largest volcano known in the Solar System: about 22 km high and some 600 km across.", NASA),
            fact("Valles Marineris, a canyon system about 4,000 km long and up to 7 km deep, would stretch across the United States.", NASA),
            fact("Mars is red because iron minerals in its dust have oxidised — it is, in effect, rusty.", NASA),
            fact("Water once flowed on the surface: Curiosity has explored ancient lake beds in Gale crater, and Perseverance is exploring a former river delta in Jezero crater.", JPL),
            fact("NASA's Ingenuity made the first powered, controlled flight on another planet on 19 April 2021 and went on to fly 72 times.", JPL),
            fact("Dust storms can grow to cover the whole planet; one in 2018 ended the Opportunity rover's 15-year mission.", JPL),
            fact("Its polar caps hold water ice, topped in winter by a layer of frozen carbon dioxide.", NASA),
            fact("MAVEN measured the solar wind stripping gas from the upper atmosphere, a process thought to have turned a thicker early atmosphere into today's thin one.", "NASA MAVEN mission"),
        ],
        ideas: &[
            idea(Open, "Was there ever life on Mars? In 2025 NASA reported that a rock Perseverance sampled in 2024 ('Cheyava Falls') carries features that could be a biosignature. Non-biological explanations have not been ruled out, and confirming it would likely require bringing the sample to Earth."),
            idea(Debated, "Is there liquid water underground today? Radar under the south polar cap shows bright reflections first read as briny lakes, which others attribute to clays or other materials; InSight's seismic data have been interpreted as water-soaked rock 11–20 km deep."),
            idea(Open, "The methane puzzle: Curiosity measures methane rising and falling with the seasons in Gale crater, while the ExoMars Trace Gas Orbiter finds almost none higher in the atmosphere."),
            idea(Debated, "Did an ocean once fill the northern lowlands? Possible ancient shorelines and tsunami deposits support it; others think the features have different origins."),
        ],
        missions: &["Viking 1 & 2 (NASA, 1976)", "Mars Reconnaissance Orbiter (NASA, 2006–)", "Curiosity (NASA, 2012–)", "MAVEN (NASA, 2014–)", "Perseverance & Ingenuity (NASA, 2021–)", "Tianwen-1 & Zhurong (China, 2021)", "Hope (UAE, 2021–)"],
    },
    Body {
        id: "phobos",
        name: "Phobos",
        kind: Kind::Moon,
        parent: Some("mars"),
        tagline: "A doomed little moon spiralling slowly into Mars.",
        radius_km: 13.5,
        motion: Motion::Moon { a_km: 9_376.0, period_days: 0.318_91, phase: 40.0, incl: 1.1 },
        spin: Spin::Locked,
        look: Look {
            color: [0.36, 0.33, 0.3],
            color2: [0.25, 0.23, 0.21],
            shape: [1.0, 0.67, 0.81],
            bumpy: 0.18,
            dot: [0.7, 0.65, 0.6],
            ..Look::ROCK
        },
        stats: &[
            ("Size", "27 × 22 × 18 km"),
            ("Distance from Mars", "9,376 km from the centre (about 6,000 km up)"),
            ("Orbit", "7 h 39 min"),
            ("Discovered", "1877, Asaph Hall"),
        ],
        facts: &[
            fact("Phobos circles Mars three times a day — faster than Mars rotates — so from the surface it rises in the west and sets in the east.", NASA),
            fact("Tides are pulling it inward by about 1.8 m per century; in roughly 50 million years it will crash into Mars or break up into a ring.", NASA),
            fact("Its largest crater, Stickney, is about 9 km wide — nearly half the moon's width.", NASA),
            fact("Its name means 'fear' in Greek; Deimos means 'dread'. Both were discovered by Asaph Hall in August 1877.", NASA),
        ],
        ideas: &[
            idea(Debated, "Where did Mars's moons come from? Their dark, asteroid-like surfaces suggest captured asteroids, but their circular, equatorial orbits fit better with moons that formed from debris after a giant impact on Mars. JAXA's Martian Moons eXploration (MMX) mission plans to bring back a sample of Phobos to settle it."),
            idea(Debated, "The long grooves across Phobos may be stretch marks from Mars's tides, or chains of debris thrown up by impacts."),
        ],
        missions: &["Mariner 9 and Viking orbiters (NASA, 1970s)", "Mars Express flybys (ESA)", "Planned: MMX (JAXA), to land on Phobos and return a sample"],
    },
    Body {
        id: "deimos",
        name: "Deimos",
        kind: Kind::Moon,
        parent: Some("mars"),
        tagline: "Mars's smaller, smoother outer moon.",
        radius_km: 7.5,
        motion: Motion::Moon { a_km: 23_463.0, period_days: 1.262_44, phase: 200.0, incl: 1.8 },
        spin: Spin::Locked,
        look: Look {
            color: [0.45, 0.41, 0.36],
            color2: [0.33, 0.3, 0.27],
            shape: [1.0, 0.69, 0.81],
            bumpy: 0.12,
            craters: 0.4,
            dot: [0.7, 0.65, 0.6],
            ..Look::ROCK
        },
        stats: &[
            ("Size", "15 × 12 × 11 km"),
            ("Distance from Mars", "23,460 km from the centre"),
            ("Orbit", "30.3 hours"),
            ("Discovered", "1877, Asaph Hall"),
        ],
        facts: &[
            fact("Deimos is one of the smallest moons in the Solar System, about 15 km across at its longest.", NASA),
            fact("Its surface looks smoother than Phobos's because a blanket of loose dust and rock fills in many of its craters.", NASA),
            fact("Unlike Phobos, Deimos orbits beyond the point where its orbital period matches Mars's day, so it is very slowly drifting away.", NASA),
        ],
        ideas: &[idea(Debated, "Like Phobos, it may be a captured asteroid or reassembled debris from an ancient impact on Mars; the two moons may not share the same origin.")],
        missions: &["Viking 2 close flyby (NASA, 1977)", "Hope orbiter close-up images (UAE, 2023)"],
    },
    // ------------------------------------------------------- asteroid belt
    Body {
        id: "asteroid-belt",
        name: "Asteroid Belt",
        kind: Kind::Region,
        parent: Some("sun"),
        tagline: "Leftovers of planet formation, between Mars and Jupiter — far emptier than films suggest.",
        radius_km: 0.0,
        motion: Motion::Region { at: (35.0, 0.0, 2.75), view: (200.0, 35.0, 12.0) },
        spin: Spin::None,
        look: Look { dot: [0.85, 0.75, 0.6], ..Look::NONE },
        stats: &[
            ("Location", "about 2.1–3.3 AU from the Sun"),
            ("Total mass", "about 3% of the Moon's"),
            ("Known asteroids", "more than 1.4 million (all regions)"),
            ("Largest member", "Ceres, about a quarter of the belt's mass"),
        ],
        facts: &[
            fact("All the asteroids in the main belt together add up to only about 3% of the Moon's mass, and nearly half of that is in Ceres, Vesta, Pallas and Hygiea.", NASA),
            fact("Asteroids are so spread out that spacecraft cross the belt without needing to dodge anything: typical spacing is on the order of a million kilometres.", NASA),
            fact("The gaps in the belt shown here (Kirkwood gaps) sit where an asteroid would orbit exactly 3, 5/2, 7/3 or 2 times for each orbit of Jupiter; repeated tugs from Jupiter clear those orbits out.", JPL),
            fact("Two swarms of 'Trojan' asteroids share Jupiter's orbit, 60° ahead of and behind the planet — also shown here. NASA's Lucy mission is on its way to visit them.", NASA),
            fact("Fragments of asteroids fall to Earth as most meteorites; OSIRIS-REx returned 121.6 g from asteroid Bennu in 2023.", NASA),
        ],
        ideas: &[
            idea(Leading, "Why is there no planet here? Jupiter's strong gravity stirred up the region, so collisions shattered bodies instead of building them up."),
            idea(Debated, "The 'Grand Tack': young Jupiter may have migrated inward to about Mars's orbit and then back out, scattering and emptying the belt. It would also explain why Mars is so small, but other models can too."),
            idea(Speculative, "Asteroid 16 Psyche may be the exposed metal core of a shattered protoplanet. NASA's Psyche spacecraft arrives in 2029 to check."),
        ],
        missions: &["Dawn (NASA, Vesta and Ceres)", "Hayabusa2 (JAXA, Ryugu sample)", "OSIRIS-REx (NASA, Bennu sample)", "Lucy (NASA, Trojans)", "Psyche (NASA, arriving 2029)"],
    },
    Body {
        id: "ceres",
        name: "Ceres",
        kind: Kind::DwarfPlanet,
        parent: Some("sun"),
        tagline: "The largest object in the asteroid belt, with salty deposits that glitter on its dark surface.",
        radius_km: 469.7,
        motion: Motion::Orbit(OrbitSpec::Classical { a: 2.767_5, e: 0.078_5, i: 10.59, node: 80.27, peri: 73.6, m: 162.0, epoch: 2025.0 }),
        spin: Spin::Iau { ra: 291.418, dec: 66.764, w0: 170.65, rate: 952.153_2 },
        look: Look {
            feature: feature::CERES,
            color: [0.42, 0.41, 0.4],
            color2: [0.33, 0.32, 0.31],
            craters: 0.8,
            dot: [0.8, 0.78, 0.75],
            ..Look::ROCK
        },
        stats: &[
            ("Mean radius", "about 470 km"),
            ("Distance from Sun", "2.77 AU"),
            ("Year", "4.6 Earth years"),
            ("Day", "9 h 4 min"),
            ("Discovered", "1 January 1801, Giuseppe Piazzi"),
        ],
        facts: &[
            fact("Ceres was the first asteroid ever found, on 1 January 1801. In 2006 the IAU reclassified it as a dwarf planet.", IAU),
            fact("It holds about a quarter of the mass of the entire asteroid belt.", NASA),
            fact("The bright spots in Occator crater are mostly sodium carbonate — salts left behind when briny water reached the surface and evaporated.", DAWN),
            fact("Ahuna Mons, a lone mountain about 4 km high, is thought to be an ice volcano (cryovolcano).", DAWN),
            fact("NASA's Dawn orbited Ceres from 2015 to 2018 — the first spacecraft to orbit a dwarf planet, and the first to orbit two different worlds beyond Earth.", DAWN),
        ],
        ideas: &[
            idea(Supported, "A reservoir of brine may still lie about 40 km beneath Occator crater, left from a deeper ocean that has mostly frozen. Dawn's gravity measurements support it."),
            idea(Debated, "Ammonia-bearing clays on the surface suggest Ceres formed farther out, in the cold outer Solar System, and later moved inward."),
        ],
        missions: &["Dawn (NASA, orbited 2015–2018)"],
    },
    Body {
        id: "vesta",
        name: "Vesta",
        kind: Kind::Asteroid,
        parent: Some("sun"),
        tagline: "A battered protoplanet whose fragments land on Earth as meteorites.",
        radius_km: 286.0,
        motion: Motion::Orbit(OrbitSpec::Classical { a: 2.361_5, e: 0.089_4, i: 7.14, node: 103.8, peri: 151.2, m: 40.0, epoch: 2025.0 }),
        spin: Spin::Iau { ra: 309.031, dec: 42.235, w0: 285.39, rate: 1_617.332_942_8 },
        look: Look {
            color: [0.55, 0.52, 0.48],
            color2: [0.4, 0.38, 0.35],
            shape: [1.0, 0.78, 0.975],
            bumpy: 0.06,
            dot: [0.85, 0.82, 0.78],
            ..Look::ROCK
        },
        stats: &[
            ("Size", "573 × 557 × 446 km"),
            ("Distance from Sun", "2.36 AU"),
            ("Year", "3.6 Earth years"),
            ("Day", "5 h 20 min"),
            ("Discovered", "1807, Heinrich Olbers"),
        ],
        facts: &[
            fact("Vesta is the second most massive object in the asteroid belt, and has a layered interior with an iron core, like the rocky planets.", DAWN),
            fact("The Rheasilvia impact basin near its south pole is about 500 km wide, almost as wide as Vesta itself, and its central peak rises about 20 km — among the tallest mountains in the Solar System.", DAWN),
            fact("The HED meteorites — around 5% of meteorites that fall on Earth — come from Vesta, blasted off by impacts like the one that made Rheasilvia.", NASA),
            fact("Dawn orbited Vesta from 2011 to 2012 before leaving for Ceres.", DAWN),
        ],
        ideas: &[idea(Leading, "Vesta is a surviving protoplanet: an early building block of the planets that melted and separated into layers, but never grew larger.")],
        missions: &["Dawn (NASA, orbited 2011–2012)"],
    },
    // -------------------------------------------------------------- Jupiter
    Body {
        id: "jupiter",
        name: "Jupiter",
        kind: Kind::Planet,
        parent: Some("sun"),
        tagline: "The giant: more than twice the mass of all the other planets combined.",
        radius_km: 71_492.0,
        motion: Motion::Orbit(OrbitSpec::Jpl(
            [5.202_887_00, 0.048_386_24, 1.304_396_95, 34.396_440_51, 14.728_479_83, 100.473_909_09],
            [-0.000_116_07, -0.000_132_53, -0.001_837_14, 3_034.746_127_75, 0.212_526_68, 0.204_691_06],
        )),
        spin: Spin::Iau { ra: 268.056_595, dec: 64.495_303, w0: 284.95, rate: 870.536 },
        look: Look {
            shade: Shade::Gas,
            tex: Some("jupiter"),
            shape: [1.0, 0.935, 1.0],
            atmosphere: [0.9, 0.8, 0.65, 0.35],
            rings: Some(Rings { inner: 1.29, outer: 1.81, tex: None, color: [0.6, 0.5, 0.42], opacity: 0.12 }),
            dot: [1.0, 0.9, 0.75],
            ..Look::ROCK
        },
        stats: &[
            ("Equatorial radius", "71,492 km (11.2 Earths)"),
            ("Mass", "318 Earths"),
            ("Distance from Sun", "5.2 AU (778 million km)"),
            ("Year", "11.86 Earth years"),
            ("Day", "9 h 56 min — the shortest of any planet"),
            ("Axial tilt", "3.1°"),
            ("Moons", "95+ known"),
        ],
        facts: &[
            fact("Jupiter is more than twice as massive as all the other planets put together.", NASA),
            fact("The Great Red Spot is a storm larger than Earth that has been watched since at least 1831 (and perhaps 1665). It has been shrinking for over a century.", NASA),
            fact("It spins once in under 10 hours, the fastest of any planet, which flattens it visibly at the poles.", NASA_FACTS),
            fact("Deep inside, hydrogen is squeezed into a liquid metal; currents in it make the strongest planetary magnetic field in the Solar System.", NASA),
            fact("Juno's gravity measurements show Jupiter's core is not a compact ball but 'dilute', mixed with hydrogen and spread over a large fraction of the planet.", JUNO),
            fact("Jupiter has faint, dusty rings, discovered by Voyager 1 in 1979.", NASA),
            fact("In July 1994 fragments of comet Shoemaker–Levy 9 hit Jupiter — the first collision between two Solar System bodies ever observed.", NASA),
            fact("Jupiter gives off about 1.6 times as much energy as it receives from the Sun, heat left over from its formation.", NASA),
        ],
        ideas: &[
            idea(Debated, "How did the core become 'dilute'? A head-on collision with a large protoplanet could have stirred it up; or Jupiter may have formed that way, with solids dissolving as it grew."),
            idea(Leading, "Core accretion: Jupiter began as a rock-and-ice core of about 10 Earth masses that then pulled in gas from the disc. The rival idea — direct collapse of part of the disc — is considered less likely for Jupiter."),
            idea(Debated, "Is Jupiter Earth's shield? It flings many comets out of the Solar System, but it also sends some inward; simulations disagree on whether it helps or hurts overall."),
        ],
        missions: &["Pioneer 10 & 11 (NASA, flybys 1973–74)", "Voyager 1 & 2 (NASA, flybys 1979)", "Galileo (NASA, orbited 1995–2003)", "Juno (NASA, orbiting since 2016)", "JUICE (ESA, launched 2023, arriving 2031)", "Europa Clipper (NASA, launched 2024, arriving 2030)"],
    },
    Body {
        id: "io",
        name: "Io",
        kind: Kind::Moon,
        parent: Some("jupiter"),
        tagline: "The most volcanically active world known.",
        radius_km: 1_821.6,
        motion: Motion::Moon { a_km: 421_700.0, period_days: 1.769_138, phase: 0.0, incl: 0.04 },
        spin: Spin::Locked,
        look: Look {
            feature: feature::IO,
            color: [0.86, 0.78, 0.36],
            color2: [0.72, 0.52, 0.24],
            craters: 0.0,
            dot: [1.0, 0.92, 0.55],
            ..Look::ROCK
        },
        stats: &[
            ("Mean radius", "1,821.6 km (a little larger than the Moon)"),
            ("Distance from Jupiter", "421,700 km"),
            ("Orbit", "1.77 days"),
            ("Discovered", "1610, Galileo Galilei"),
        ],
        facts: &[
            fact("Io has hundreds of active volcanoes — around 400 — some throwing plumes of sulfur and sulfur dioxide hundreds of kilometres high.", NASA),
            fact("Its heat comes from tides: an orbital resonance with Europa and Ganymede (Io orbits 4 times for every 2 of Europa's and 1 of Ganymede's) keeps Io's orbit slightly oval, so Jupiter squeezes and relaxes it every 1.77 days.", NASA),
            fact("Loki Patera is a lava lake about 200 km across.", NASA),
            fact("Io has no impact craters visible: eruptions resurface it faster than craters can build up.", NASA),
            fact("Volcanic gas escaping Io fills a doughnut of charged particles (the Io plasma torus) around Jupiter.", NASA),
        ],
        ideas: &[idea(Debated, "Is there a global magma ocean just under the crust? Galileo's magnetic data were read that way in 2011, but Juno's precise gravity measurements, published in 2024, argue against a shallow magma ocean and favour a mostly solid mantle.")],
        missions: &["Voyager 1 discovered active volcanism (1979)", "Galileo (NASA, 1995–2003)", "Juno close flybys (NASA, 2023–2024)"],
    },
    Body {
        id: "europa",
        name: "Europa",
        kind: Kind::Moon,
        parent: Some("jupiter"),
        tagline: "An ice shell over a hidden ocean — one of the best places to look for life.",
        radius_km: 1_560.8,
        motion: Motion::Moon { a_km: 671_034.0, period_days: 3.551_181, phase: 100.0, incl: 0.47 },
        spin: Spin::Locked,
        look: Look {
            feature: feature::EUROPA,
            color: [0.88, 0.84, 0.76],
            color2: [0.6, 0.42, 0.28],
            craters: 0.0,
            dot: [0.95, 0.92, 0.85],
            ..Look::ROCK
        },
        stats: &[
            ("Mean radius", "1,560.8 km"),
            ("Distance from Jupiter", "671,034 km"),
            ("Orbit", "3.55 days"),
            ("Discovered", "1610, Galileo Galilei"),
        ],
        facts: &[
            fact("Galileo's magnetometer found that Europa disturbs Jupiter's magnetic field the way a salty, electrically conducting ocean would — strong evidence of a global ocean under the ice.", GALILEO),
            fact("That ocean may hold about twice as much water as all of Earth's oceans.", NASA),
            fact("Its surface is among the youngest in the Solar System, perhaps 40–90 million years old, judging by how few craters it has.", NASA),
            fact("The reddish-brown cracks (lineae) crisscrossing the ice are thought to be fractures where material from below has welled up.", NASA),
            fact("NASA's Europa Clipper launched in October 2024 and is due to arrive in 2030 to make about 50 close flybys.", JPL),
        ],
        ideas: &[
            idea(Open, "Could Europa's ocean support life? It probably has water, chemical energy and the right elements; nobody knows whether life ever started there."),
            idea(Disputed, "Water plumes: Hubble saw possible plumes in 2012 and 2016, and Galileo data were later reinterpreted as passing through one, but repeated searches have not confirmed them."),
            idea(Debated, "How thick is the ice? Estimates run from a few kilometres to over 25 km, which matters for whether ocean material can reach the surface."),
        ],
        missions: &["Galileo (NASA, 1995–2003)", "Juno flyby (NASA, 2022)", "Europa Clipper (NASA, arriving 2030)", "JUICE flybys (ESA, 2030s)"],
    },
    Body {
        id: "ganymede",
        name: "Ganymede",
        kind: Kind::Moon,
        parent: Some("jupiter"),
        tagline: "The largest moon in the Solar System — bigger than Mercury.",
        radius_km: 2_634.1,
        motion: Motion::Moon { a_km: 1_070_412.0, period_days: 7.154_553, phase: 220.0, incl: 0.2 },
        spin: Spin::Locked,
        look: Look {
            feature: feature::GANYMEDE,
            color: [0.62, 0.58, 0.52],
            color2: [0.36, 0.32, 0.28],
            craters: 0.6,
            dot: [0.85, 0.8, 0.75],
            ..Look::ROCK
        },
        stats: &[
            ("Mean radius", "2,634.1 km (larger than Mercury)"),
            ("Distance from Jupiter", "1,070,412 km"),
            ("Orbit", "7.15 days"),
            ("Discovered", "1610, Galileo Galilei"),
        ],
        facts: &[
            fact("Ganymede is the largest moon in the Solar System, wider than the planet Mercury, though only about half as massive.", NASA),
            fact("It is the only moon known to make its own magnetic field, discovered by Galileo in 1996.", GALILEO),
            fact("Hubble watched Ganymede's auroras rock back and forth less than expected, indicating a salty ocean beneath the ice that damps them (2015).", "NASA / ESA Hubble"),
            fact("ESA's JUICE is due to enter orbit around Ganymede in 2034, the first spacecraft to orbit a moon other than our own.", ESA),
        ],
        ideas: &[idea(Debated, "Ganymede's ocean may be sandwiched between layers of different kinds of ice, or it may sit directly on rock, which would make it more favourable for chemistry that could support life.")],
        missions: &["Galileo (NASA, 1995–2003)", "Juno flyby (NASA, 2021)", "JUICE (ESA, orbit from 2034)"],
    },
    Body {
        id: "callisto",
        name: "Callisto",
        kind: Kind::Moon,
        parent: Some("jupiter"),
        tagline: "An ancient, unchanging surface covered in craters.",
        radius_km: 2_410.3,
        motion: Motion::Moon { a_km: 1_882_709.0, period_days: 16.689_018, phase: 310.0, incl: 0.2 },
        spin: Spin::Locked,
        look: Look {
            feature: feature::CALLISTO,
            color: [0.3, 0.27, 0.24],
            color2: [0.55, 0.52, 0.48],
            craters: 1.2,
            dot: [0.7, 0.65, 0.6],
            ..Look::ROCK
        },
        stats: &[
            ("Mean radius", "2,410.3 km"),
            ("Distance from Jupiter", "1,882,709 km"),
            ("Orbit", "16.7 days"),
            ("Discovered", "1610, Galileo Galilei"),
        ],
        facts: &[
            fact("Callisto is the most heavily cratered object known in the Solar System; its surface has changed little in about 4 billion years.", NASA),
            fact("Valhalla, its largest impact scar, is a bright centre surrounded by rings spreading roughly 3,800 km across.", NASA),
            fact("It is outside the orbital resonance that heats Io, Europa and Ganymede, so it receives little tidal heating.", NASA),
        ],
        ideas: &[
            idea(Debated, "Galileo's magnetic data hint at a salty ocean deep beneath the surface, but the signal could also come from Jupiter's surrounding plasma."),
            idea(Supported, "Callisto seems only partly separated into layers — rock and ice remain partly mixed — suggesting it formed slowly and never melted completely."),
        ],
        missions: &["Voyager 1 & 2 (NASA, 1979)", "Galileo (NASA, 1995–2003)", "JUICE flybys (ESA, 2030s)"],
    },
    // --------------------------------------------------------------- Saturn
    Body {
        id: "saturn",
        name: "Saturn",
        kind: Kind::Planet,
        parent: Some("sun"),
        tagline: "The ringed giant — so light for its size it would float in water.",
        radius_km: 60_268.0,
        motion: Motion::Orbit(OrbitSpec::Jpl(
            [9.536_675_94, 0.053_861_79, 2.485_991_87, 49.954_244_23, 92.598_878_31, 113.662_424_48],
            [-0.001_250_60, -0.000_509_91, 0.001_936_09, 1_222.493_622_01, -0.418_972_16, -0.288_677_94],
        )),
        spin: Spin::Iau { ra: 40.589, dec: 83.537, w0: 38.90, rate: 810.793_902_4 },
        look: Look {
            shade: Shade::Gas,
            tex: Some("saturn"),
            shape: [1.0, 0.902, 1.0],
            atmosphere: [0.95, 0.85, 0.6, 0.3],
            rings: Some(Rings { inner: 1.166, outer: 2.327, tex: Some("saturn_ring_alpha"), color: [0.85, 0.78, 0.65], opacity: 1.0 }),
            dot: [1.0, 0.92, 0.7],
            ..Look::ROCK
        },
        stats: &[
            ("Equatorial radius", "60,268 km (9.4 Earths)"),
            ("Mass", "95 Earths"),
            ("Distance from Sun", "9.5 AU (1.43 billion km)"),
            ("Year", "29.5 Earth years"),
            ("Day", "10 h 33 min"),
            ("Axial tilt", "26.7°"),
            ("Mean density", "0.69 g/cm³ (water is 1.0)"),
            ("Moons", "274 known (2025)"),
        ],
        facts: &[
            fact("Saturn's average density is lower than water's — the only planet for which that is true.", NASA_FACTS),
            fact("The main rings span about 270,000 km but are typically only about 10 m thick.", NASA),
            fact("The rings are almost entirely water ice, in pieces from dust grains to chunks the size of houses.", NASA),
            fact("A six-sided jet stream circles the north pole; each side of the hexagon is longer than Earth's diameter.", CASSINI),
            fact("In March 2025 astronomers announced 128 newly discovered moons, bringing Saturn's total to 274 — more than every other planet combined.", IAU),
            fact("Earth passed through Saturn's ring plane on 23 March 2025, so the rings appeared edge-on and almost vanished from view.", NASA),
            fact("Cassini orbited Saturn for 13 years and ended its mission by diving into the planet on 15 September 2017.", CASSINI),
        ],
        ideas: &[
            idea(Debated, "How old are the rings? Cassini found them so clean of dust that they might be only a few hundred million years old; more recent studies argue that old rings could also stay clean."),
            idea(Speculative, "The rings may be the remains of a destroyed moon. One 2022 model ('Chrysalis') proposes that a lost moon both made the rings and explains Saturn's tilt."),
            idea(Leading, "The hexagon is a stable wave in a jet stream; similar shapes form in laboratory tanks of spinning fluid."),
        ],
        missions: &["Pioneer 11 (NASA, flyby 1979)", "Voyager 1 & 2 (NASA, flybys 1980–81)", "Cassini–Huygens (NASA/ESA/ASI, 2004–2017)", "Planned: Dragonfly to Titan (NASA)"],
    },
    Body {
        id: "mimas",
        name: "Mimas",
        kind: Kind::Moon,
        parent: Some("saturn"),
        tagline: "The 'Death Star' moon, scarred by one enormous crater.",
        radius_km: 198.2,
        motion: Motion::Moon { a_km: 185_539.0, period_days: 0.942_422, phase: 30.0, incl: 1.57 },
        spin: Spin::Locked,
        look: Look {
            feature: feature::MIMAS,
            color: [0.78, 0.77, 0.75],
            color2: [0.62, 0.61, 0.6],
            craters: 1.1,
            dot: [0.9, 0.9, 0.88],
            ..Look::ROCK
        },
        stats: &[
            ("Mean radius", "198.2 km"),
            ("Distance from Saturn", "185,539 km"),
            ("Orbit", "22.6 hours"),
            ("Discovered", "1789, William Herschel"),
        ],
        facts: &[
            fact("Herschel crater is about 139 km wide — roughly a third of Mimas's diameter — giving it a famous resemblance to the Death Star.", NASA),
            fact("Mimas's gravity clears the Cassini Division, the largest gap in Saturn's rings, through a 2:1 orbital resonance.", NASA),
            fact("Cassini's thermal maps showed a 'Pac-Man' temperature pattern on Mimas, caused by high-energy electrons hardening the surface ice.", CASSINI),
        ],
        ideas: &[idea(Debated, "A 2024 study of Mimas's slight wobble and orbit suggests a young global ocean, perhaps only 5–15 million years old, beneath 20–30 km of ice. If confirmed, it would mean even small, inactive-looking moons can hide oceans.")],
        missions: &["Voyager 1 (NASA, 1980)", "Cassini (closest flyby 2010)"],
    },
    Body {
        id: "enceladus",
        name: "Enceladus",
        kind: Kind::Moon,
        parent: Some("saturn"),
        tagline: "A small, brilliant moon spraying its hidden ocean into space.",
        radius_km: 252.1,
        motion: Motion::Moon { a_km: 237_948.0, period_days: 1.370_218, phase: 160.0, incl: 0.01 },
        spin: Spin::Locked,
        look: Look {
            feature: feature::ENCELADUS,
            color: [0.95, 0.96, 0.98],
            color2: [0.82, 0.86, 0.9],
            craters: 0.35,
            dot: [1.0, 1.0, 1.0],
            ..Look::ROCK
        },
        stats: &[
            ("Mean radius", "252.1 km"),
            ("Distance from Saturn", "237,948 km"),
            ("Orbit", "32.9 hours"),
            ("Discovered", "1789, William Herschel"),
        ],
        facts: &[
            fact("Jets of water vapour and ice burst from four cracks near the south pole (the 'tiger stripes'), feeding Saturn's E ring.", CASSINI),
            fact("Cassini measured a slight wobble in Enceladus's rotation that shows a global ocean lies beneath the ice shell.", CASSINI),
            fact("Tiny silica grains in the plume point to hydrothermal activity — hot water reacting with rock on the ocean floor.", CASSINI),
            fact("Cassini detected molecular hydrogen and organic molecules in the plume, and in 2023 scientists reported phosphates in ice grains from it.", CASSINI),
            fact("Its fresh ice makes it one of the most reflective objects in the Solar System.", NASA),
        ],
        ideas: &[
            idea(Open, "Is Enceladus's ocean inhabited? It appears to have liquid water, energy and the key chemical ingredients; whether life exists there is unknown. It is a leading target for future missions."),
            idea(Debated, "How long has the ocean existed? If it is old and stable, life would have had more time to arise; some models suggest the activity could be relatively recent."),
        ],
        missions: &["Voyager 2 (NASA, 1981)", "Cassini (NASA/ESA/ASI, 23 close flybys, 2005–2015)"],
    },
    Body {
        id: "titan",
        name: "Titan",
        kind: Kind::Moon,
        parent: Some("saturn"),
        tagline: "A moon with a thick atmosphere, methane rain and seas.",
        radius_km: 2_574.7,
        motion: Motion::Moon { a_km: 1_221_870.0, period_days: 15.945, phase: 260.0, incl: 0.35 },
        spin: Spin::Locked,
        look: Look {
            feature: feature::TITAN,
            color: [0.85, 0.6, 0.28],
            color2: [0.7, 0.45, 0.2],
            craters: 0.0,
            atmosphere: [1.0, 0.7, 0.35, 1.1],
            dot: [1.0, 0.8, 0.5],
            ..Look::ROCK
        },
        stats: &[
            ("Mean radius", "2,574.7 km (larger than Mercury)"),
            ("Distance from Saturn", "1,221,870 km"),
            ("Orbit", "15.9 days"),
            ("Surface pressure", "about 1.5 times Earth's"),
            ("Surface temperature", "about −179 °C"),
            ("Discovered", "1655, Christiaan Huygens"),
        ],
        facts: &[
            fact("Titan is the only moon with a thick atmosphere — mostly nitrogen, like Earth's, with surface pressure about 1.5 times ours.", NASA),
            fact("It has rain, rivers, lakes and seas, but of liquid methane and ethane. Kraken Mare, the largest sea, is bigger than the Caspian Sea.", CASSINI),
            fact("ESA's Huygens probe landed on Titan on 14 January 2005 — still the most distant landing ever made.", ESA),
            fact("It is the second-largest moon in the Solar System, after Ganymede.", NASA),
            fact("NASA's Dragonfly, a nuclear-powered rotorcraft, is due to launch in 2028 and arrive in 2034 to fly between sites on Titan's surface.", NASA),
        ],
        ideas: &[
            idea(Debated, "Cassini measured Titan flexing under Saturn's tides, long read as evidence of a subsurface water ocean; a 2025 reanalysis argued the data fit a slushy, partly frozen layer better."),
            idea(Speculative, "Could life use liquid methane instead of water? Chemists have proposed exotic cell membranes that could work in Titan's seas, but whether such chemistry is possible is unknown."),
        ],
        missions: &["Voyager 1 (NASA, 1980)", "Cassini–Huygens (NASA/ESA/ASI, 2004–2017)", "Dragonfly (NASA, arriving 2034)"],
    },
    // --------------------------------------------------------------- Uranus
    Body {
        id: "uranus",
        name: "Uranus",
        kind: Kind::Planet,
        parent: Some("sun"),
        tagline: "The ice giant that rolls around the Sun on its side.",
        radius_km: 25_559.0,
        motion: Motion::Orbit(OrbitSpec::Jpl(
            [19.189_164_64, 0.047_257_44, 0.772_637_83, 313.238_104_51, 170.954_276_30, 74.016_925_03],
            [-0.001_961_76, -0.000_043_97, -0.002_429_39, 428.482_027_85, 0.408_052_81, 0.042_405_89],
        )),
        spin: Spin::Iau { ra: 257.311, dec: -15.175, w0: 203.81, rate: -501.160_092_8 },
        look: Look {
            shade: Shade::Gas,
            tex: Some("uranus"),
            shape: [1.0, 0.977, 1.0],
            atmosphere: [0.6, 0.9, 0.95, 0.5],
            rings: Some(Rings { inner: 1.64, outer: 2.0, tex: None, color: [0.55, 0.6, 0.65], opacity: 0.35 }),
            dot: [0.7, 0.95, 1.0],
            ..Look::ROCK
        },
        stats: &[
            ("Equatorial radius", "25,559 km (4.0 Earths)"),
            ("Mass", "14.5 Earths"),
            ("Distance from Sun", "19.2 AU (2.9 billion km)"),
            ("Year", "84 Earth years"),
            ("Day", "17 h 14 min, backwards"),
            ("Axial tilt", "97.8°"),
            ("Moons", "29 known (2025)"),
        ],
        facts: &[
            fact("Uranus's axis is tilted about 98°, so it orbits on its side; each pole gets around 42 years of continuous sunlight followed by 42 years of darkness.", NASA),
            fact("William Herschel discovered it in 1781 — the first planet found with a telescope.", NASA),
            fact("It is an 'ice giant': beneath its hydrogen–helium atmosphere is a dense fluid of water, methane and ammonia. Methane gas absorbs red light, giving it its blue-green colour.", NASA),
            fact("It has 13 known rings, the first discovered in 1977 when they blocked the light of a background star.", NASA),
            fact("Voyager 2 is the only spacecraft to have visited, flying past in January 1986.", VOYAGER),
            fact("Its atmosphere is the coldest measured of any planet, reaching about −224 °C.", NASA),
        ],
        ideas: &[
            idea(Leading, "What knocked Uranus over? The leading idea is one or more giant collisions with Earth-sized bodies during its formation; an alternative involves slow gravitational interactions with a former large moon."),
            idea(Supported, "Voyager 2 may have seen Uranus at an unusual moment: a 2024 reanalysis found the solar wind was strongly compressed during the flyby, distorting the magnetosphere we measured."),
            idea(Open, "Why does Uranus give off so little internal heat compared with Neptune? Its interior may be layered in a way that traps heat, or the giant impact may have drained it."),
        ],
        missions: &["Voyager 2 (NASA, flyby January 1986)", "Proposed: Uranus Orbiter and Probe — top flagship priority of the 2022 U.S. planetary science decadal survey"],
    },
    Body {
        id: "miranda",
        name: "Miranda",
        kind: Kind::Moon,
        parent: Some("uranus"),
        tagline: "A patchwork moon with one of the tallest cliffs known.",
        radius_km: 235.8,
        motion: Motion::Moon { a_km: 129_390.0, period_days: 1.413_479, phase: 70.0, incl: 4.3 },
        spin: Spin::Locked,
        look: Look {
            feature: feature::MIRANDA,
            color: [0.72, 0.71, 0.7],
            color2: [0.5, 0.5, 0.5],
            craters: 0.6,
            dot: [0.85, 0.85, 0.85],
            ..Look::ROCK
        },
        stats: &[
            ("Mean radius", "235.8 km"),
            ("Distance from Uranus", "129,390 km"),
            ("Orbit", "1.41 days"),
            ("Discovered", "1948, Gerard Kuiper"),
        ],
        facts: &[
            fact("Verona Rupes, a fault cliff, is estimated to be up to about 20 km high — possibly the tallest cliff known in the Solar System.", NASA),
            fact("Its surface is a jumble of old cratered terrain and huge grooved, racetrack-shaped regions called coronae.", NASA),
            fact("Everything we know about Miranda's surface comes from Voyager 2's single flyby in 1986, which imaged only its southern hemisphere.", VOYAGER),
        ],
        ideas: &[
            idea(Debated, "The coronae may have formed as warm ice rose from within, driven by tidal heating when Miranda's orbit was more eccentric. An older idea — that Miranda was smashed apart and reassembled — is now less favoured."),
            idea(Speculative, "A 2024 study modelling Miranda's surface features suggested it may have had a liquid ocean beneath its crust in the recent geological past, and might still retain part of it."),
        ],
        missions: &["Voyager 2 (NASA, 1986)"],
    },
    // -------------------------------------------------------------- Neptune
    Body {
        id: "neptune",
        name: "Neptune",
        kind: Kind::Planet,
        parent: Some("sun"),
        tagline: "The windiest planet, found by mathematics before it was seen.",
        radius_km: 24_764.0,
        motion: Motion::Orbit(OrbitSpec::Jpl(
            [30.069_922_76, 0.008_590_48, 1.770_043_47, -55.120_029_69, 44.964_762_27, 131.784_225_74],
            [0.000_262_91, 0.000_051_05, 0.000_353_72, 218.459_453_25, -0.322_414_64, -0.005_086_64],
        )),
        spin: Spin::Iau { ra: 299.36, dec: 43.46, w0: 249.978, rate: 541.139_775_7 },
        look: Look {
            shade: Shade::Gas,
            tex: Some("neptune"),
            shape: [1.0, 0.983, 1.0],
            atmosphere: [0.35, 0.55, 1.0, 0.5],
            rings: Some(Rings { inner: 1.67, outer: 2.55, tex: None, color: [0.45, 0.45, 0.5], opacity: 0.12 }),
            dot: [0.45, 0.65, 1.0],
            ..Look::ROCK
        },
        stats: &[
            ("Equatorial radius", "24,764 km (3.9 Earths)"),
            ("Mass", "17.1 Earths"),
            ("Distance from Sun", "30.1 AU (4.5 billion km)"),
            ("Year", "164.8 Earth years"),
            ("Day", "16 h 6 min"),
            ("Axial tilt", "28.3°"),
            ("Moons", "16 known"),
        ],
        facts: &[
            fact("Neptune was the first planet found by prediction: Urbain Le Verrier calculated its position from Uranus's wobbles, and Johann Galle saw it on 23 September 1846, within about 1° of the prediction.", NASA),
            fact("Its winds are the fastest measured on any planet, reaching about 2,000 km/h.", NASA),
            fact("Neptune takes 165 years to orbit the Sun; in 2011 it completed its first full orbit since its discovery.", NASA),
            fact("Voyager 2 photographed a Great Dark Spot in 1989; when Hubble looked in 1994, it had vanished. Other dark spots have come and gone since.", "NASA / ESA Hubble"),
            fact("Neptune radiates about 2.6 times as much energy as it receives from the Sun.", NASA),
            fact("Voyager 2 is the only spacecraft to have visited, flying past in August 1989.", VOYAGER),
        ],
        ideas: &[
            idea(Leading, "Neptune and Uranus probably formed closer to the Sun and migrated outward, scattering smaller bodies as they went (as in the 'Nice model'). This would explain the structure of the Kuiper belt."),
            idea(Supported, "'Diamond rain': at the pressures inside the ice giants, carbon from methane may be squeezed into diamonds that sink toward the core. Laser experiments on Earth have produced nanodiamonds under similar conditions."),
            idea(Open, "Why does Neptune have so much more internal heat than Uranus, even though the two planets are near-twins in size and composition?"),
        ],
        missions: &["Voyager 2 (NASA, flyby August 1989)"],
    },
    Body {
        id: "triton",
        name: "Triton",
        kind: Kind::Moon,
        parent: Some("neptune"),
        tagline: "A captured world orbiting backwards, with nitrogen geysers.",
        radius_km: 1_353.4,
        motion: Motion::Moon { a_km: 354_759.0, period_days: 5.876_854, phase: 90.0, incl: 157.3 },
        spin: Spin::Locked,
        look: Look {
            feature: feature::TRITON,
            color: [0.86, 0.8, 0.76],
            color2: [0.72, 0.6, 0.58],
            craters: 0.0,
            atmosphere: [0.6, 0.7, 0.9, 0.2],
            dot: [0.95, 0.88, 0.85],
            ..Look::ROCK
        },
        stats: &[
            ("Mean radius", "1,353.4 km"),
            ("Distance from Neptune", "354,759 km"),
            ("Orbit", "5.88 days, backwards"),
            ("Surface temperature", "about −235 °C"),
            ("Discovered", "1846, William Lassell"),
        ],
        facts: &[
            fact("Triton is the only large moon in the Solar System that orbits in the opposite direction to its planet's rotation.", NASA),
            fact("Voyager 2 saw dark plumes rising about 8 km above its surface — geysers driven by sunlight warming nitrogen ice.", VOYAGER),
            fact("At about −235 °C, its surface is one of the coldest ever measured in the Solar System.", NASA),
            fact("William Lassell discovered Triton just 17 days after Neptune itself was found.", NASA),
        ],
        ideas: &[
            idea(Leading, "Triton was probably a dwarf planet from the Kuiper belt, captured by Neptune — which explains its backward orbit. The capture would have disrupted Neptune's original moons."),
            idea(Supported, "Tides are slowly dragging Triton inward; in billions of years it is expected to be torn apart, perhaps forming a ring."),
            idea(Speculative, "Triton may hold a subsurface ocean warmed by tidal heating left over from its capture."),
        ],
        missions: &["Voyager 2 (NASA, 1989)"],
    },
    // ---------------------------------------------------------------- Pluto
    Body {
        id: "pluto",
        name: "Pluto",
        kind: Kind::DwarfPlanet,
        parent: Some("sun"),
        tagline: "A small world with a heart of nitrogen ice, glaciers and blue skies.",
        radius_km: 1_188.3,
        motion: Motion::Orbit(OrbitSpec::Jpl(
            [39.482_116_75, 0.248_827_30, 17.140_012_06, 238.929_038_33, 224.068_916_29, 110.303_936_84],
            [-0.000_315_96, 0.000_051_70, 0.000_048_18, 145.207_805_15, -0.040_629_42, -0.011_834_82],
        )),
        spin: Spin::Iau { ra: 132.993, dec: -6.163, w0: 302.695, rate: -56.362_522_5 },
        look: Look {
            feature: feature::PLUTO,
            color: [0.78, 0.62, 0.48],
            color2: [0.35, 0.2, 0.14],
            craters: 0.12,
            atmosphere: [0.55, 0.7, 1.0, 0.25],
            dot: [0.95, 0.85, 0.75],
            ..Look::ROCK
        },
        stats: &[
            ("Mean radius", "1,188.3 km (0.19 Earth)"),
            ("Mass", "0.0022 Earth"),
            ("Distance from Sun", "39.5 AU average (29.7–49.3 AU)"),
            ("Year", "248 Earth years"),
            ("Day", "6.4 Earth days, backwards"),
            ("Surface temperature", "about −230 °C"),
            ("Moons", "5"),
            ("Discovered", "18 February 1930, Clyde Tombaugh"),
        ],
        facts: &[
            fact("Clyde Tombaugh discovered Pluto in 1930 at Lowell Observatory. In 2006 the IAU defined 'planet' in a way that reclassified Pluto as a dwarf planet.", IAU),
            fact("New Horizons flew past on 14 July 2015, the first spacecraft to visit.", NEW_HORIZONS),
            fact("The bright 'heart', Tombaugh Regio, includes Sputnik Planitia: a basin of nitrogen ice about 1,000 km wide that slowly churns in convection cells.", NEW_HORIZONS),
            fact("Pluto has mountains of water ice several kilometres high — at −230 °C, water ice is as hard as rock.", NEW_HORIZONS),
            fact("Its thin nitrogen atmosphere has layers of haze that scatter sunlight blue.", NEW_HORIZONS),
            fact("Pluto and its largest moon Charon always show each other the same face: both are tidally locked.", NASA),
            fact("Pluto's orbit crosses Neptune's, but a 3:2 resonance (Pluto orbits twice for every three Neptune orbits) means they can never collide.", NASA),
        ],
        ideas: &[
            idea(Supported, "A liquid water ocean may lie beneath Pluto's ice. Sputnik Planitia's position — almost exactly opposite Charon — suggests extra mass beneath it, which a dense subsurface ocean would supply."),
            idea(Debated, "Wright Mons and Piccard Mons, broad mountains with central depressions, may be ice volcanoes (cryovolcanoes) that erupted relatively recently."),
            idea(Open, "Will Pluto's atmosphere freeze onto the surface as it moves away from the Sun? Models predicted a collapse; observations of stars passing behind Pluto have not yet seen it happen."),
        ],
        missions: &["New Horizons (NASA, flyby 14 July 2015)"],
    },
    Body {
        id: "charon",
        name: "Charon",
        kind: Kind::Moon,
        parent: Some("pluto"),
        tagline: "Pluto's partner, half its size, with a red-stained pole.",
        radius_km: 606.0,
        motion: Motion::Moon { a_km: 19_591.0, period_days: 6.387_2, phase: 180.0, incl: 0.0 },
        spin: Spin::Locked,
        look: Look {
            feature: feature::CHARON,
            color: [0.58, 0.57, 0.56],
            color2: [0.45, 0.25, 0.18],
            craters: 0.6,
            dot: [0.8, 0.78, 0.76],
            ..Look::ROCK
        },
        stats: &[
            ("Mean radius", "606 km (about half of Pluto)"),
            ("Distance from Pluto", "19,591 km"),
            ("Orbit", "6.39 days (the same as Pluto's day)"),
            ("Discovered", "1978, James Christy"),
        ],
        facts: &[
            fact("Charon is so large relative to Pluto that the two orbit a point in space between them; they are often described as a double dwarf planet.", NASA),
            fact("Its dark red north polar region, nicknamed Mordor Macula, is thought to be methane escaped from Pluto, caught by Charon and turned into reddish compounds by ultraviolet light.", NEW_HORIZONS),
            fact("The James Webb Space Telescope detected carbon dioxide and hydrogen peroxide on Charon's surface (2024).", "NASA / ESA / CSA JWST"),
        ],
        ideas: &[idea(Leading, "Charon formed from a giant collision between Pluto and another large Kuiper belt object, much as Earth's Moon is thought to have formed.")],
        missions: &["New Horizons (NASA, 2015)"],
    },
    // ---------------------------------------------------------- Kuiper belt
    Body {
        id: "kuiper-belt",
        name: "Kuiper Belt",
        kind: Kind::Region,
        parent: Some("sun"),
        tagline: "A vast ring of icy bodies beyond Neptune: the Solar System's frozen outskirts.",
        radius_km: 0.0,
        motion: Motion::Region { at: (120.0, 0.0, 44.0), view: (250.0, 32.0, 90.0) },
        spin: Spin::None,
        look: Look { dot: [0.7, 0.8, 1.0], ..Look::NONE },
        stats: &[
            ("Location", "about 30–50 AU from the Sun"),
            ("Known objects", "thousands catalogued"),
            ("First found after Pluto", "1992, (15760) Albion"),
            ("Largest members", "Pluto, Eris (scattered disc), Haumea, Makemake"),
        ],
        facts: &[
            fact("The Kuiper belt is a doughnut-shaped region of icy bodies beyond Neptune's orbit, roughly 30 to 50 AU from the Sun.", NASA),
            fact("David Jewitt and Jane Luu found the first Kuiper belt object after Pluto in 1992, now named Albion.", MPC),
            fact("Many short-period comets, such as those that return every few decades, are thought to come from the Kuiper belt and the scattered disc beyond it.", NASA),
            fact("Pluto and the 'plutinos' share a 3:2 resonance with Neptune at about 39.4 AU; the densest part of the belt, the 'classical' belt, lies between about 42 and 48 AU.", MPC),
            fact("The belt seems to end abruptly near 50 AU (the 'Kuiper cliff').", NASA),
        ],
        ideas: &[
            idea(Open, "What causes the sharp outer edge? It may mark the edge of the original planet-forming disc, or something — a passing star or an unseen planet — may have cleared the region beyond."),
            idea(Speculative, "New Horizons has measured more dust than expected far out, and deep surveys have found distant objects that may hint at a second, more distant belt. Both are unconfirmed."),
            idea(Leading, "Neptune's outward migration early in the Solar System's history pushed many objects into resonances and scattered others, sculpting the belt we see."),
        ],
        missions: &["New Horizons (NASA, Pluto 2015, Arrokoth 2019)"],
    },
    Body {
        id: "arrokoth",
        name: "Arrokoth",
        kind: Kind::Tno,
        parent: Some("sun"),
        tagline: "The most distant object ever explored up close: a pristine snowman of the early Solar System.",
        radius_km: 18.0,
        motion: Motion::Orbit(OrbitSpec::Classical { a: 44.58, e: 0.034, i: 2.45, node: 159.0, peri: 175.0, m: 312.5, epoch: 2025.0 }),
        spin: Spin::Simple { hours: 15.92 },
        look: Look {
            color: [0.62, 0.36, 0.26],
            color2: [0.48, 0.27, 0.2],
            craters: 0.2,
            bumpy: 0.05,
            lobes: &[
                Lobe { offset: [-0.43, 0.0, 0.0], scale: [0.57, 0.26, 0.55] },
                Lobe { offset: [0.57, 0.0, 0.0], scale: [0.43, 0.27, 0.38] },
            ],
            dot: [0.9, 0.6, 0.5],
            ..Look::ROCK
        },
        stats: &[
            ("Size", "36 km long"),
            ("Distance from Sun", "about 44.6 AU"),
            ("Year", "about 298 Earth years"),
            ("Day", "15.9 hours"),
            ("Visited", "1 January 2019, New Horizons"),
        ],
        facts: &[
            fact("On 1 January 2019, New Horizons flew 3,500 km from Arrokoth, about 6.6 billion km from Earth — the most distant object ever explored up close.", NEW_HORIZONS),
            fact("Arrokoth is a 'contact binary': two flattened lobes, about 21 km and 15 km across, gently joined.", NEW_HORIZONS),
            fact("Its red colour comes from organic compounds called tholins, made as radiation alters ices such as methane.", NEW_HORIZONS),
            fact("Its name comes from the Powhatan/Algonquian language and means 'sky'.", NEW_HORIZONS),
        ],
        ideas: &[idea(Supported, "Arrokoth's undamaged, gently merged lobes support the idea that planetesimals formed by the gentle gravitational collapse of clouds of pebbles, rather than by violent collisions.")],
        missions: &["New Horizons (NASA, flyby 2019)"],
    },
    Body {
        id: "haumea",
        name: "Haumea",
        kind: Kind::DwarfPlanet,
        parent: Some("sun"),
        tagline: "A dwarf planet stretched into an egg by its fast spin — with a ring.",
        radius_km: 1_161.0,
        motion: Motion::Orbit(OrbitSpec::Classical { a: 43.1, e: 0.195, i: 28.2, node: 121.8, peri: 239.0, m: 223.0, epoch: 2025.0 }),
        spin: Spin::Simple { hours: 3.915 },
        look: Look {
            feature: feature::ICY,
            color: [0.9, 0.9, 0.92],
            color2: [0.62, 0.42, 0.42],
            craters: 0.2,
            shape: [1.0, 0.44, 0.734],
            rings: Some(Rings { inner: 1.92, outer: 2.02, tex: None, color: [0.8, 0.8, 0.82], opacity: 0.4 }),
            dot: [0.95, 0.95, 0.97],
            ..Look::ROCK
        },
        stats: &[
            ("Size", "about 2,320 × 1,700 × 1,030 km"),
            ("Distance from Sun", "43 AU average"),
            ("Year", "about 284 Earth years"),
            ("Day", "3.9 hours"),
            ("Moons", "2 (Hiʻiaka and Namaka)"),
        ],
        facts: &[
            fact("Haumea spins once every 3.9 hours, the fastest of any known large body in the Solar System, which stretches it into an elongated shape.", NASA),
            fact("A ring was discovered around it in 2017 when it passed in front of a star — the first ring found around a trans-Neptunian object.", "Ortiz et al., Nature (2017)"),
            fact("Its surface is mostly crystalline water ice, with a dark red spot.", NASA),
            fact("Its moons are named after daughters of the Hawaiian goddess Haumea.", IAU),
        ],
        ideas: &[idea(Leading, "A giant collision long ago probably spun Haumea up and blasted off icy fragments — its moons and a 'family' of similar icy objects on related orbits.")],
        missions: &["None yet: studied by telescopes"],
    },
    Body {
        id: "makemake",
        name: "Makemake",
        kind: Kind::DwarfPlanet,
        parent: Some("sun"),
        tagline: "A bright, reddish dwarf planet discovered around Easter 2005.",
        radius_km: 715.0,
        motion: Motion::Orbit(OrbitSpec::Classical { a: 45.43, e: 0.161, i: 28.98, node: 79.62, peri: 297.24, m: 163.4, epoch: 2025.0 }),
        spin: Spin::Simple { hours: 22.83 },
        look: Look {
            feature: feature::ICY,
            color: [0.82, 0.62, 0.48],
            color2: [0.62, 0.42, 0.3],
            craters: 0.15,
            dot: [0.95, 0.8, 0.7],
            ..Look::ROCK
        },
        stats: &[
            ("Diameter", "about 1,430 km"),
            ("Distance from Sun", "45 AU average"),
            ("Year", "about 305 Earth years"),
            ("Day", "22.8 hours"),
            ("Moons", "1 known"),
            ("Discovered", "2005, Michael Brown's team"),
        ],
        facts: &[
            fact("Makemake was found shortly after Easter 2005 and is named after the creator god of the Rapa Nui people of Easter Island.", IAU),
            fact("Its surface is covered in frozen methane, which gives it a reddish-brown tint.", NASA),
            fact("When it passed in front of a star in 2011, the starlight vanished and returned abruptly, showing it has no substantial global atmosphere.", "Ortiz et al., Nature (2012)"),
            fact("Hubble discovered a small, dark moon around it in 2016.", "NASA / ESA Hubble"),
        ],
        ideas: &[idea(Open, "Makemake may have localised, temporary atmosphere or activity; how volatile ices move around on its surface is still being studied.")],
        missions: &["None yet: studied by telescopes"],
    },
    Body {
        id: "eris",
        name: "Eris",
        kind: Kind::DwarfPlanet,
        parent: Some("sun"),
        tagline: "The discovery that cost Pluto its planet status.",
        radius_km: 1_163.0,
        motion: Motion::Orbit(OrbitSpec::Classical { a: 67.86, e: 0.436, i: 44.04, node: 35.95, peri: 151.6, m: 210.6, epoch: 2025.0 }),
        spin: Spin::Simple { hours: 378.9 },
        look: Look {
            feature: feature::ICY,
            color: [0.93, 0.93, 0.94],
            color2: [0.8, 0.79, 0.8],
            craters: 0.15,
            dot: [1.0, 1.0, 1.0],
            ..Look::ROCK
        },
        stats: &[
            ("Diameter", "about 2,326 km (almost Pluto's size)"),
            ("Mass", "about 1.27 Pluto"),
            ("Distance from Sun", "about 96 AU now (38–98 AU)"),
            ("Year", "about 559 Earth years"),
            ("Moons", "1 (Dysnomia)"),
            ("Discovered", "2005, Brown, Trujillo and Rabinowitz"),
        ],
        facts: &[
            fact("Eris is almost the same size as Pluto but about 27% more massive. Its discovery in 2005 led the IAU to define the word 'planet' in 2006.", IAU),
            fact("It is currently about 96 AU from the Sun, near the farthest point of its orbit and around three times farther than Pluto.", JPL),
            fact("Its orbit is tilted 44° to the plane of the planets.", MPC),
            fact("It is named after the Greek goddess of strife and discord; its moon Dysnomia after her daughter, the spirit of lawlessness.", IAU),
        ],
        ideas: &[idea(Open, "Eris reflects about 96% of the light that hits it. Its atmosphere may currently be frozen onto the surface as fresh frost, and might partly thaw when it swings in to 38 AU in about 250 years.")],
        missions: &["None yet: studied by telescopes"],
    },
    // --------------------------------------------------------------- comets
    Body {
        id: "halley",
        name: "Halley's Comet",
        kind: Kind::Comet,
        parent: Some("sun"),
        tagline: "The famous comet that returns every 75–79 years, on a backwards orbit.",
        radius_km: 7.5,
        motion: Motion::Orbit(OrbitSpec::Classical { a: 17.83, e: 0.967, i: 162.26, node: 58.42, peri: 111.33, m: 185.9, epoch: 2025.0 }),
        spin: Spin::Simple { hours: 52.8 },
        look: Look {
            color: [0.22, 0.2, 0.19],
            color2: [0.15, 0.14, 0.13],
            shape: [1.0, 0.53, 0.53],
            bumpy: 0.22,
            craters: 0.3,
            dot: [0.75, 0.9, 1.0],
            ..Look::ROCK
        },
        stats: &[
            ("Nucleus", "about 15 × 8 km"),
            ("Orbit", "about 76 years"),
            ("Closest to Sun", "0.59 AU"),
            ("Farthest from Sun", "about 35 AU (December 2023)"),
            ("Last perihelion", "9 February 1986"),
            ("Next perihelion", "28 July 2061"),
        ],
        facts: &[
            fact("Edmond Halley calculated in 1705 that comets seen in 1531, 1607 and 1682 were the same object, and predicted its return in 1758. It was the first comet known to be periodic.", NASA),
            fact("Halley orbits the Sun backwards compared with the planets, on an orbit tilted 162°.", JPL),
            fact("ESA's Giotto flew within about 600 km of the nucleus in 1986 and found it darker than coal, reflecting only about 4% of sunlight.", ESA),
            fact("Dust shed by Halley causes two meteor showers each year: the Eta Aquariids in May and the Orionids in October.", NASA),
            fact("It reached the farthest point of its orbit in December 2023 and is now on its way back, due at perihelion on 28 July 2061.", JPL),
        ],
        ideas: &[idea(Debated, "Halley-type comets may have come from the Oort cloud and been captured into shorter orbits by the planets; others argue for the scattered disc beyond Neptune as their source.")],
        missions: &["Giotto (ESA, 1986)", "Vega 1 & 2 (USSR, 1986)", "Suisei and Sakigake (Japan, 1986)"],
    },
    // ------------------------------------------------------------ spacecraft
    Body {
        id: "new-horizons",
        name: "New Horizons",
        kind: Kind::Spacecraft,
        parent: Some("sun"),
        tagline: "The spacecraft that showed us Pluto and Arrokoth, still exploring the Kuiper belt.",
        radius_km: 30.0,
        motion: Motion::Probe { lon: 284.0, lat: 2.5, au_2025: 61.0, au_per_year: 2.9, launched: 2006.05 },
        spin: Spin::FaceSun,
        look: Look::NEW_HORIZONS,
        stats: &[
            ("Launched", "19 January 2006"),
            ("Speed", "about 14 km/s"),
            ("Pluto flyby", "14 July 2015"),
            ("Arrokoth flyby", "1 January 2019"),
        ],
        facts: &[
            fact("New Horizons left Earth faster than any spacecraft before it, at about 58,500 km/h relative to Earth.", NEW_HORIZONS),
            fact("It reached Pluto in 9.5 years, using a gravity assist from Jupiter in 2007.", NEW_HORIZONS),
            fact("It carries some of the ashes of Clyde Tombaugh, who discovered Pluto.", NEW_HORIZONS),
            fact("Its nuclear power source is expected to keep it working into the 2040s.", NEW_HORIZONS),
        ],
        ideas: &[idea(Open, "Its dust counter keeps seeing more dust than models predict this far out, which might hint at undiscovered Kuiper belt objects.")],
        missions: &["Position shown is approximate"],
    },
    Body {
        id: "voyager-2",
        name: "Voyager 2",
        kind: Kind::Spacecraft,
        parent: Some("sun"),
        tagline: "The only spacecraft to have visited Uranus and Neptune.",
        radius_km: 30.0,
        motion: Motion::Probe { lon: 290.0, lat: -38.0, au_2025: 138.0, au_per_year: 3.25, launched: 1977.63 },
        spin: Spin::FaceSun,
        look: Look::VOYAGER,
        stats: &[
            ("Launched", "20 August 1977"),
            ("Speed", "about 15.4 km/s"),
            ("Planets visited", "Jupiter, Saturn, Uranus, Neptune"),
            ("Entered interstellar space", "5 November 2018"),
        ],
        facts: &[
            fact("Voyager 2 is the only spacecraft to have visited Uranus (1986) and Neptune (1989).", VOYAGER),
            fact("It launched 16 days before Voyager 1, on a slower route that let it visit all four giant planets.", VOYAGER),
            fact("It crossed the heliopause into interstellar space on 5 November 2018, at about 119 AU.", VOYAGER),
            fact("Like Voyager 1, it carries a gold-plated record of sounds and images from Earth.", VOYAGER),
        ],
        ideas: &[idea(Open, "How long can it keep working? Engineers switch off instruments and heaters one by one to stretch its fading nuclear power; it may keep at least one instrument running into the 2030s.")],
        missions: &["Position shown is approximate"],
    },
    // ---------------------------------------------------------- heliosphere
    Body {
        id: "heliosphere",
        name: "Heliosphere",
        kind: Kind::Region,
        parent: Some("sun"),
        tagline: "The bubble blown by the solar wind, and its edge — the heliopause — where interstellar space begins.",
        radius_km: 0.0,
        motion: Motion::Region { at: (255.4, 5.2, 121.0), view: (345.0, 35.0, 1_500.0) },
        spin: Spin::None,
        look: Look { dot: [1.0, 0.75, 0.5], ..Look::NONE },
        stats: &[
            ("Termination shock", "about 84–94 AU"),
            ("Heliopause", "about 119–122 AU (where the Voyagers crossed)"),
            ("Solar wind speed", "about 400 km/s typical"),
        ],
        facts: &[
            fact("The Sun blows a constant wind of charged particles that carves out a bubble in the gas between the stars: the heliosphere.", NASA),
            fact("At the termination shock the solar wind abruptly slows below the speed of sound; Voyager 1 crossed it at 94 AU in 2004 and Voyager 2 at 84 AU in 2007.", VOYAGER),
            fact("Voyager 1 crossed the heliopause into interstellar space on 25 August 2012 at about 121.6 AU; Voyager 2 followed in 2018 at about 119 AU.", VOYAGER),
            fact("Three objects from other star systems have been seen passing through: 1I/ʻOumuamua (2017), comet 2I/Borisov (2019) and comet 3I/ATLAS (2025).", "IAU Minor Planet Center / NASA"),
            fact("NASA's IMAP, launched in September 2025, is mapping the heliosphere's boundary from near Earth.", NASA),
        ],
        ideas: &[
            idea(Debated, "What shape is the heliosphere? The classic picture is a comet-like bubble with a long tail; some models and data suggest a shorter, rounder or even croissant-like shape. The shells drawn here are illustrative."),
            idea(Debated, "What was ʻOumuamua? It sped up slightly as it left, like a comet, but showed no visible gas or dust. The leading explanation is gas, such as hydrogen, escaping from its ice; claims that it was an alien craft are not supported by the evidence."),
        ],
        missions: &["Voyager 1 & 2 (NASA, in situ)", "IBEX (NASA, 2008–)", "IMAP (NASA, 2025–)"],
    },
    Body {
        id: "voyager-1",
        name: "Voyager 1",
        kind: Kind::Spacecraft,
        parent: Some("sun"),
        tagline: "The most distant human-made object, travelling through interstellar space.",
        radius_km: 30.0,
        motion: Motion::Probe { lon: 256.0, lat: 35.0, au_2025: 165.0, au_per_year: 3.57, launched: 1977.68 },
        spin: Spin::FaceSun,
        look: Look::VOYAGER,
        stats: &[
            ("Launched", "5 September 1977"),
            ("Speed", "about 17 km/s (3.6 AU per year)"),
            ("Planets visited", "Jupiter (1979), Saturn (1980)"),
            ("Entered interstellar space", "25 August 2012"),
        ],
        facts: &[
            fact("Voyager 1 is the most distant human-made object; radio signals take nearly a day to reach it. It is expected to be one light-day from Earth around November 2026.", VOYAGER),
            fact("It was the first spacecraft to enter interstellar space, crossing the heliopause on 25 August 2012.", VOYAGER),
            fact("On 14 February 1990 it turned around and photographed Earth from about 6 billion km away: the 'Pale Blue Dot'.", JPL),
            fact("It carries the Golden Record, with greetings in 55 languages, music and sounds of Earth.", VOYAGER),
            fact("Even so, it will take about 300 years to reach the inner edge of the Oort cloud and perhaps 30,000 years to pass through it.", NASA),
        ],
        ideas: &[idea(Open, "Its power supply is slowly fading; engineers have been switching off instruments to keep it going, and it may fall silent around 2030.")],
        missions: &["Position shown is approximate"],
    },
    // ---------------------------------------------------------- far outskirts
    Body {
        id: "sedna",
        name: "Sedna",
        kind: Kind::Tno,
        parent: Some("sun"),
        tagline: "A red world on an enormous orbit that never comes near the planets.",
        radius_km: 500.0,
        motion: Motion::Orbit(OrbitSpec::Classical { a: 506.0, e: 0.849, i: 11.93, node: 144.2, peri: 311.3, m: 358.4, epoch: 2025.0 }),
        spin: Spin::Simple { hours: 10.3 },
        look: Look {
            feature: feature::ICY,
            color: [0.72, 0.32, 0.22],
            color2: [0.55, 0.24, 0.18],
            craters: 0.2,
            dot: [1.0, 0.5, 0.4],
            ..Look::ROCK
        },
        stats: &[
            ("Diameter", "about 1,000 km (estimated)"),
            ("Closest to Sun", "about 76 AU (around 2076)"),
            ("Farthest from Sun", "over 900 AU"),
            ("Year", "about 11,000 years"),
            ("Discovered", "2003, Brown, Trujillo and Rabinowitz"),
        ],
        facts: &[
            fact("Sedna's orbit takes it from about 76 AU to more than 900 AU from the Sun; one lap takes roughly 11,000 years.", JPL),
            fact("It never comes closer than about 76 AU, so Neptune's gravity cannot explain how it got onto this orbit.", NASA),
            fact("It is one of the reddest objects known in the Solar System.", NASA),
            fact("It reaches its closest point to the Sun around 2076 — a rare chance to study it.", JPL),
        ],
        ideas: &[idea(Debated, "What put Sedna on its detached orbit? Leading ideas: a star that passed close to the young Sun in its birth cluster, or the gravity of an undiscovered distant planet.")],
        missions: &["None: studied by telescopes"],
    },
    Body {
        id: "planet-nine",
        name: "Planet Nine?",
        kind: Kind::Hypothetical,
        parent: Some("sun"),
        tagline: "A proposed planet far beyond Neptune. It may not exist — this position is illustrative only.",
        radius_km: 16_000.0,
        motion: Motion::Fixed { lon: 53.0, lat: -9.0, au: 460.0 },
        spin: Spin::Simple { hours: 20.0 },
        look: Look { shade: Shade::Ghost, color: [0.55, 0.75, 1.0], dot: [0.55, 0.75, 1.0], ..Look::ROCK },
        stats: &[
            ("Status", "Hypothetical — not observed"),
            ("Proposed", "2016, Konstantin Batygin and Michael Brown"),
            ("Predicted mass", "about 5–10 Earths"),
            ("Predicted orbit", "a few hundred AU from the Sun"),
        ],
        facts: &[
            fact("In 2014 Chad Trujillo and Scott Sheppard noticed that several of the most distant known objects have strangely similar orbits.", "Trujillo & Sheppard, Nature (2014)"),
            fact("In 2016 Konstantin Batygin and Michael Brown proposed that an unseen planet of several Earth masses could herd those orbits together.", "Batygin & Brown, Astronomical Journal (2016)"),
            fact("No telescope has detected Planet Nine. The Vera C. Rubin Observatory, which released its first images in 2025, is expected to find it or rule out much of where it could be.", "Rubin Observatory"),
        ],
        ideas: &[
            idea(Debated, "Does it exist? Some surveys argue the orbital clustering is an artefact of where telescopes happened to look (observational bias), which would remove the need for a planet."),
            idea(Speculative, "Alternatives to a planet include the combined gravity of a massive disc of small icy bodies, or even a primordial black hole of similar mass."),
            idea(Speculative, "If it exists, it might be the core of a giant planet ejected from the young Solar System, or a planet captured from another star."),
        ],
        missions: &["Searches: Vera C. Rubin Observatory's Legacy Survey of Space and Time"],
    },
    Body {
        id: "oort-cloud",
        name: "Oort Cloud",
        kind: Kind::Region,
        parent: Some("sun"),
        tagline: "A giant, unseen shell of icy bodies that may stretch a third of the way to the nearest star.",
        radius_km: 0.0,
        motion: Motion::Region { at: (30.0, 25.0, 6_000.0), view: (140.0, 28.0, 90_000.0) },
        spin: Spin::None,
        look: Look { dot: [0.75, 0.85, 1.0], ..Look::NONE },
        stats: &[
            ("Inner edge", "about 2,000–5,000 AU (estimated)"),
            ("Outer edge", "perhaps 100,000 AU — 1.6 light-years (estimated)"),
            ("Objects", "possibly hundreds of billions or more"),
            ("Ever seen directly?", "No"),
        ],
        facts: &[
            fact("No Oort cloud object has ever been observed in place: the cloud is inferred from the orbits of long-period comets, which fall in from all directions.", NASA),
            fact("Jan Oort proposed the cloud in 1950 to explain where those comets come from.", NASA),
            fact("Even at 17 km/s, Voyager 1 will need about 300 years to reach its inner edge and perhaps 30,000 years to cross it.", NASA),
            fact("At its outer edge, the Sun's gravity is so weak that passing stars and the tide of the Milky Way itself can nudge comets onto paths toward the inner Solar System.", NASA),
        ],
        ideas: &[
            idea(Leading, "The Oort cloud exists, formed from icy planetesimals flung outward by the giant planets early in the Solar System's history. This is widely accepted, though unproven by direct observation."),
            idea(Speculative, "The Sun may have captured a large share of its Oort cloud comets from other stars in its birth cluster."),
            idea(Speculative, "A 2025 simulation suggested the Milky Way's tide could twist the inner Oort cloud into a spiral structure about 15,000 AU long."),
        ],
        missions: &["None can reach it within centuries"],
    },
];

/// The order of the grand tour (and of Next/Previous): the catalogue order.
pub fn tour() -> impl Iterator<Item = usize> {
    0..BODIES.len()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ids_are_unique_and_parents_come_first() {
        for (i, b) in BODIES.iter().enumerate() {
            assert_eq!(index_of(b.id), Some(i), "duplicate id {}", b.id);
            if let Some(p) = b.parent {
                let pi = index_of(p).unwrap_or_else(|| panic!("{} has unknown parent {p}", b.id));
                assert!(pi < i, "{} is listed before its parent", b.id);
            }
        }
    }

    #[test]
    fn every_body_is_annotated() {
        for b in BODIES {
            assert!(!b.facts.is_empty(), "{} has no facts", b.id);
            assert!(!b.ideas.is_empty(), "{} has no hypotheses", b.id);
            assert!(!b.stats.is_empty(), "{} has no stats", b.id);
            assert!(!b.tagline.is_empty());
            for f in b.facts {
                assert!(!f.source.is_empty() && f.text.ends_with(['.', ')']), "{}: {}", b.id, f.text);
            }
            for i in b.ideas {
                assert!(i.text.ends_with(['.', '?']), "{}: {}", b.id, i.text);
            }
        }
    }

    #[test]
    fn moons_have_planet_parents() {
        for b in BODIES {
            if let Motion::Moon { .. } | Motion::Lunar = b.motion {
                assert_eq!(b.kind, Kind::Moon);
                let parent = &BODIES[index_of(b.parent.unwrap()).unwrap()];
                assert!(matches!(parent.kind, Kind::Planet | Kind::DwarfPlanet));
            }
        }
    }
}
