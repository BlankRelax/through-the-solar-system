# Through the Solar System

An annotated, first-person flight through the Solar System, from the Sun out to the Oort cloud.
You are a point of view, free to go anywhere; every object on the way is annotated with
**verified facts**, each with its source, and **hypotheses**, each labelled with how settled it is
(leading hypothesis, supported by evidence, debated, open question, disputed, speculative).

Written in Rust, compiled to WebAssembly, drawn with WebGL2.

- **Play it:** [blankrelax.github.io/through-the-solar-system](https://blankrelax.github.io/through-the-solar-system/),
  or on [hassaanmirza.co.uk/solar-system](https://hassaanmirza.co.uk/solar-system)
- **Offline:** download `through-the-solar-system.html` from the
  [latest release](https://github.com/BlankRelax/through-the-solar-system/releases/latest) and open
  it in any modern browser. It is one self-contained file (about 3.6 MB) with the game, its
  textures and styles inside, and needs no server or internet connection.

## What is in it

- **39 objects**: the Sun; the eight planets; the Moon, Phobos, Deimos, the four Galilean moons,
  Mimas, Enceladus, Titan, Miranda, Triton and Charon; Ceres, Vesta, Pluto, Haumea, Makemake, Eris,
  Arrokoth and Sedna; Halley's Comet; Voyager 1, Voyager 2 and New Horizons; the asteroid belt, the
  Kuiper belt, the heliosphere and the Oort cloud; and the hypothetical Planet Nine.
- **Real positions.** Planets are placed for the current date (or any date from 1800 to 2050,
  using the time controls) from NASA/JPL's approximate planetary elements, and smaller bodies from
  published orbital elements. Earth and the Moon turn to show the right face to the Sun; the Moon's
  phase is right to within a few degrees. Planets' axes and spins follow the IAU rotation models.
- **Live readouts**: each object's current distance from the Sun and from Earth, and how long light
  or radio signals take to cross it (Voyager 1 is almost a light-day away).
- **Swarms**: some 57,000 asteroids, Trojans, Kuiper belt objects and Oort cloud comets, each moving
  on its own orbit, with the Kirkwood gaps cleared by Jupiter.
- **Rendering**: real texture maps for the Sun, planets and Moon; procedural surfaces for the
  other moons and dwarf planets (Io's volcanoes, Europa's cracks, Enceladus's tiger stripes,
  Pluto's heart, Charon's red pole and more); Earth's night lights, clouds and ocean glint; ring
  shadows on Saturn and Saturn's shadow on its rings; eclipses cast by moons; atmospheres seen
  edge-on; Halley's tail when it nears the Sun; the Milky Way in its true place on the sky.
- **A journal**: fly close to an object (or arrive at a region) to chart it; progress is kept in
  the browser.

## The scale

Space is almost entirely empty. If the Sun were a beach ball 70 cm wide, Earth would be a
peppercorn 75 m away and Neptune 2.3 km away. So that everything fits, the game draws each object
in its **true direction** from the Sun but at a **squeezed distance** (60 × √(distance in AU)
display units), and draws sizes enlarged (radius ∝ R^0.55). The numbers in the panels are always
the real ones.

## Controls

| Input | Action |
| --- | --- |
| Drag | Look around, or circle the object you are visiting |
| Scroll / pinch | Zoom, or change speed in free flight |
| W A S D, R / F | Fly forward, back, left, right, up, down |
| Q / E | Roll |
| Arrow keys | Turn (or circle the object you are visiting) |
| Shift | Fly five times faster |
| Click | Select an object and read about it |
| Double-click, G, Enter | Fly to the selected object |
| [ and ] | Previous and next stop on the grand tour |
| Space | Pause or resume time |
| , and . | Slow time down or speed it up |
| T | Back to the present moment |
| O, L, J, H | Orbits, labels, journal, help |
| Esc | Close a panel, or stop and fly freely |

## Building

You need Rust (with the `wasm32-unknown-unknown` target), `wasm-bindgen-cli` at exactly the
version pinned in `Cargo.toml`, and Node.js.

```sh
rustup target add wasm32-unknown-unknown
cargo install wasm-bindgen-cli --version 0.2.129 --locked
cargo test                 # orbits, rotation, the catalogue, formatting
node scripts/build.mjs     # dist/ (a static site) and dist-single/ (one HTML file)
node scripts/check.mjs     # optional: fly around in headless Chrome, screenshots in .check/
```

Serve `dist/` with any static web server (for example `python -m http.server -d dist`); opening
`dist/index.html` straight from disk does not work, because browsers will not load WebAssembly
modules from `file://` addresses. `dist-single/through-the-solar-system.html` has everything inlined
and does open from disk.

Every push to `main` is built, tested, checked in headless Chrome and published to GitHub Pages
(`.github/workflows/build.yml`); pushing a `v*` tag also attaches the single-file build to a
GitHub release.

## Embedding

The game runs inside any element that has a size:

```html
<link rel="stylesheet" href="tss.css" />
<div id="game" style="height: 600px"></div>
<script type="module">
  import init, { start } from './pkg/through_the_solar_system.js';
  await init();
  start(document.getElementById('game'), { assets: './textures/' });
</script>
```

Pages with a Content-Security-Policy need `'wasm-unsafe-eval'` in `script-src`. The game sets
styles only through the CSSOM and loads nothing from other origins.

## How it is put together

```
src/
  astro.rs      time, Kepler orbits, IAU rotation, reference frames, the compressed scale
  catalog.rs    every object: motion, look, figures, facts (with sources), hypotheses (with status)
  sim.rs        where everything is, and which way it faces, at a moment
  belts.rs      the asteroid belt, Trojans, Kuiper belt, scattered disc and Oort cloud particles
  mesh.rs       spheres, ring strips, the background stars, the heliosphere shells
  camera.rs     the point of view
  format.rs     numbers and light-travel times for the readouts
  web/          the browser side (WebAssembly only)
    mod.rs        start(): canvas, WebGL2 context, events, the frame loop
    app.rs        free flight, circling, autopilot, picking, the journal, readouts
    render.rs     draws a frame
    ui.rs         labels, readout, controls, panel, journal and help (HTML around the canvas)
    gl.rs         small WebGL2 helpers
    shaders/      GLSL ES 3.0: planets, rings, atmospheres, sky, stars, swarms, orbits, glows
web/            the standalone page, the game's stylesheet, and the texture maps
scripts/        build.mjs (the web build), check.mjs (the browser check)
```

The simulation and catalogue are plain Rust with native unit tests (`cargo test`), which check,
among other things, Earth's position on 1 January 2000, the date of a full Moon, and the current
distances of Eris, Sedna, Makemake, Haumea and Halley's Comet.

### Adding or correcting an annotation

Each object is one `Body` in `src/catalog.rs`. Facts are `fact(text, source)`; hypotheses are
`idea(status, text)`, where the status is one of `Leading`, `Supported`, `Debated`, `Open`,
`Disputed` or `Speculative`. The tests require every object to have figures, at least one fact
with a source and at least one hypothesis.

## Accuracy

Facts were written from NASA, ESA, JPL, IAU and peer-reviewed sources and phrased conservatively;
hypotheses are labelled by how settled they are, and the field moves quickly. Positions of the
planets are good to well under a degree for 1800–2050; moon positions (other than our Moon's)
and spacecraft positions are approximate, and Planet Nine's position is illustrative only, since
it may not exist. Corrections are welcome as issues or pull requests.

## Credits and licence

Code: MIT (see `LICENSE`). Texture maps: [Solar System Scope](https://www.solarsystemscope.com/textures/),
CC BY 4.0, based on NASA mission data (see `TEXTURES.md`). Orbital elements: NASA/JPL (E. M.
Standish) and published elements for smaller bodies. Rotation models: IAU Working Group on
Cartographic Coordinates and Rotational Elements.
