// Hash-based gradient noise, fractal sums and craters, all in 3D so surfaces
// have no seams or pinched poles.

vec3 hash33(vec3 p) {
  p = fract(p * vec3(0.1031, 0.1030, 0.0973));
  p += dot(p, p.yxz + 33.33);
  return fract((p.xxy + p.yxx) * p.zyx) * 2.0 - 1.0;
}

float gnoise(vec3 p) {
  vec3 i = floor(p);
  vec3 f = fract(p);
  vec3 u = f * f * (3.0 - 2.0 * f);
  return mix(
    mix(mix(dot(hash33(i), f), dot(hash33(i + vec3(1, 0, 0)), f - vec3(1, 0, 0)), u.x),
        mix(dot(hash33(i + vec3(0, 1, 0)), f - vec3(0, 1, 0)), dot(hash33(i + vec3(1, 1, 0)), f - vec3(1, 1, 0)), u.x), u.y),
    mix(mix(dot(hash33(i + vec3(0, 0, 1)), f - vec3(0, 0, 1)), dot(hash33(i + vec3(1, 0, 1)), f - vec3(1, 0, 1)), u.x),
        mix(dot(hash33(i + vec3(0, 1, 1)), f - vec3(0, 1, 1)), dot(hash33(i + vec3(1, 1, 1)), f - vec3(1, 1, 1)), u.x), u.y),
    u.z);
}

float fbm(vec3 p) {
  float sum = 0.0;
  float amp = 0.5;
  for (int i = 0; i < 5; i++) {
    sum += amp * gnoise(p);
    p = p * 2.03 + vec3(17.1, 3.7, 9.2);
    amp *= 0.5;
  }
  return sum;
}

// Distance to the nearest random feature point (cellular noise).
float cellDist(vec3 p) {
  vec3 i = floor(p);
  vec3 f = fract(p);
  float d = 8.0;
  for (int z = -1; z <= 1; z++)
    for (int y = -1; y <= 1; y++)
      for (int x = -1; x <= 1; x++) {
        vec3 g = vec3(x, y, z);
        d = min(d, length(g + 0.5 + 0.4 * hash33(i + g) - f));
      }
  return d;
}

// One layer of craters: a bowl with a raised rim around random points.
float craterLayer(vec3 p, float scale, float seed) {
  p = p * scale + seed;
  vec3 i = floor(p);
  vec3 f = fract(p);
  float h = 0.0;
  for (int z = -1; z <= 1; z++)
    for (int y = -1; y <= 1; y++)
      for (int x = -1; x <= 1; x++) {
        vec3 g = vec3(x, y, z);
        vec3 r = hash33(i + g);
        float radius = 0.12 + 0.2 * fract(r.x * 7.31 + r.y * 3.17);
        float e = length(g + 0.5 + 0.35 * r - f) / radius;
        if (e < 1.6) {
          float bowl = e < 1.0 ? (e * e - 1.0) : 0.0;
          float rim = exp(-pow((e - 1.0) * 4.0, 2.0)) * 0.4;
          h += (bowl + rim) * radius;
        }
      }
  return h;
}
