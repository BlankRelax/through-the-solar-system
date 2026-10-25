# Texture credits

The maps in `web/textures/` are from [Solar System Scope](https://www.solarsystemscope.com/textures/),
licensed under [Creative Commons Attribution 4.0 International](https://creativecommons.org/licenses/by/4.0/).
They are based on NASA mission data (and, for Earth, NASA's Blue Marble and Black Marble imagery).

They were downloaded at 2048 × 1024 and re-encoded as WebP (most reduced to 1024 × 512); the Earth
cloud map was converted to greyscale. No other changes were made.

| File | Source map |
| --- | --- |
| `sun.webp` | 2k_sun |
| `mercury.webp` | 2k_mercury |
| `venus_atmosphere.webp` | 2k_venus_atmosphere |
| `earth_daymap.webp` | 2k_earth_daymap |
| `earth_nightmap.webp` | 2k_earth_nightmap |
| `earth_clouds.webp` | 2k_earth_clouds |
| `moon.webp` | 2k_moon |
| `mars.webp` | 2k_mars |
| `jupiter.webp` | 2k_jupiter |
| `saturn.webp` | 2k_saturn |
| `saturn_ring_alpha.webp` | 2k_saturn_ring_alpha |
| `uranus.webp` | 2k_uranus |
| `neptune.webp` | 2k_neptune |
| `stars_milky_way.webp` | 2k_stars_milky_way |

All other surfaces (the moons of the giant planets, Pluto and Charon, dwarf planets, asteroids and
comets) are generated procedurally in `src/web/shaders/planet.frag` as approximations of their
known appearance.
