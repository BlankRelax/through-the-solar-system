// Surfaces: texture maps where real ones exist, procedural worlds elsewhere,
// lit by the Sun with soft shadows from moons, planets and rings.

in vec3 v_local;
in vec3 v_world;
in vec2 v_uv;
in float v_logz;

uniform mat3 u_basis;
uniform vec3 u_scale;
uniform float u_bumpy;
uniform float u_seed;
uniform int u_shade;    // 0 sun, 1 rock, 2 gas, 3 earth, 4 ghost
uniform int u_feature;  // procedural surface, see catalog::feature
uniform int u_hasTex;
uniform sampler2D u_tex0;  // surface
uniform sampler2D u_tex1;  // night lights
uniform sampler2D u_tex2;  // clouds
uniform vec3 u_color;
uniform vec3 u_color2;
uniform float u_craters;
uniform vec3 u_sun;        // Sun centre relative to the camera
uniform float u_sunRadius;
uniform vec4 u_occ[4];     // shadow casters: centre (camera-relative), radius
uniform int u_occN;
uniform vec3 u_center;
uniform vec3 u_ringNormal;
uniform vec4 u_ring;       // inner, outer radius (display units), opacity, has texture
uniform sampler2D u_ringTex;
uniform vec4 u_atmo;       // colour, strength
uniform float u_time;      // days (clouds) or seconds (the Sun)
uniform float u_logF;

out vec4 o_color;

const float PI = 3.14159265;

float angle(vec3 a, vec3 b) { return acos(clamp(dot(a, b), -1.0, 1.0)); }

vec3 dirAt(float latDeg, float lonDeg) {
  float la = radians(latDeg), lo = radians(lonDeg);
  return vec3(cos(la) * cos(lo), sin(la), -cos(la) * sin(lo));
}

float height(vec3 p) {
  float h = 0.0;
  if (u_craters > 0.0) {
    h = (craterLayer(p, 2.6, u_seed) * 0.9 + craterLayer(p, 6.0, u_seed + 3.1) * 0.5
       + craterLayer(p, 14.0, u_seed + 9.7) * 0.25) * u_craters;
  }
  if (u_feature == 7) {  // Mimas: Herschel crater, 139 km across
    float e = angle(p, dirAt(-1.7, 112.0)) / 0.35;
    if (e < 1.5) h += (e < 1.0 ? (e * e - 1.0) * 0.5 : 0.0) + exp(-pow((e - 1.0) * 5.0, 2.0)) * 0.2
                    + exp(-pow(e * 9.0, 2.0)) * 0.35;
  }
  if (u_feature == 11) {  // Ceres: Occator crater
    float e = angle(p, dirAt(19.8, 239.3)) / 0.1;
    if (e < 1.5) h += (e < 1.0 ? (e * e - 1.0) * 0.3 : 0.0) + exp(-pow((e - 1.0) * 5.0, 2.0)) * 0.12;
  }
  if (u_feature == 14) return 0.0;
  return h + fbm(p * 4.0 + u_seed) * 0.08;
}

vec3 surface(vec3 p, float h) {
  float n = fbm(p * 2.2 + u_seed);
  float n2 = fbm(p * 8.0 + u_seed * 1.3);
  float lat = asin(clamp(p.y, -1.0, 1.0));
  float lon = atan(-p.z, p.x);
  vec3 col = mix(u_color2, u_color, smoothstep(-0.35, 0.35, n + 0.35 * n2));

  if (u_feature == 1) {  // Io: sulfur plains, red rings and black volcanic pits
    col = mix(vec3(0.93, 0.85, 0.45), vec3(0.82, 0.6, 0.27), smoothstep(-0.2, 0.45, n));
    col = mix(col, vec3(0.95, 0.94, 0.8), smoothstep(0.2, 0.5, n2) * 0.45);
    col = mix(col, vec3(0.5, 0.33, 0.22), smoothstep(0.6, 1.0, abs(p.y)) * 0.7);
    float d = cellDist(p * 5.0 + u_seed);
    col = mix(col, vec3(0.78, 0.3, 0.1), smoothstep(0.34, 0.14, d) * 0.85);
    col = mix(col, vec3(0.06, 0.05, 0.04), smoothstep(0.13, 0.07, d));
  } else if (u_feature == 2) {  // Europa: ice with reddish-brown cracks
    float lines = smoothstep(0.035, 0.0, abs(gnoise(p * 3.0 + u_seed)))
                + smoothstep(0.025, 0.0, abs(gnoise(p * 6.5 + 4.0))) * 0.8
                + smoothstep(0.02, 0.0, abs(gnoise(p * 13.0 + 9.0))) * 0.6;
    vec3 ice = mix(vec3(0.92, 0.9, 0.85), vec3(0.72, 0.62, 0.52), smoothstep(0.05, 0.45, n));
    col = mix(ice, vec3(0.55, 0.32, 0.2), clamp(lines, 0.0, 1.0) * 0.8);
  } else if (u_feature == 3) {  // Ganymede: dark ancient terrain and bright grooved terrain
    float dark = smoothstep(-0.02, 0.12, fbm(p * 1.4 + u_seed));
    float grooves = 0.5 + 0.5 * sin(dot(p, normalize(vec3(1.0, 0.35, 0.6))) * 90.0 + n * 25.0);
    vec3 light = mix(vec3(0.66, 0.63, 0.58), vec3(0.8, 0.78, 0.74), grooves);
    col = mix(light, vec3(0.33, 0.29, 0.25), dark);
    col = mix(col, vec3(0.92, 0.92, 0.94), smoothstep(0.75, 0.95, abs(p.y)) * 0.5);
  } else if (u_feature == 4) {  // Callisto: dark crust peppered with bright craters
    col = mix(vec3(0.22, 0.19, 0.16), vec3(0.33, 0.29, 0.25), n * 0.5 + 0.5);
    col += vec3(0.45) * smoothstep(0.015, 0.06, h);
  } else if (u_feature == 5) {  // Titan: featureless orange haze, faint bands
    col = mix(vec3(0.8, 0.55, 0.24), vec3(0.9, 0.68, 0.33), 0.5 + 0.5 * sin(lat * 3.0 + n * 0.8));
  } else if (u_feature == 6) {  // Enceladus: fresh snow, blue 'tiger stripes' at the south pole
    col = mix(col, vec3(0.97, 0.98, 1.0), 0.5);
    float s = sin(dot(p.xz, vec2(0.8, 0.6)) * 34.0 + n * 2.0);
    col = mix(col, vec3(0.42, 0.6, 0.78), smoothstep(0.9, 1.0, s) * smoothstep(-0.9, -1.15, lat));
  } else if (u_feature == 8) {  // Triton: pinkish south polar cap, 'cantaloupe' terrain
    float cap = smoothstep(-0.05, -0.35, lat);
    vec3 north = mix(vec3(0.66, 0.68, 0.62), vec3(0.78, 0.76, 0.7), smoothstep(0.3, 0.6, cellDist(p * 9.0)));
    vec3 south = mix(vec3(0.92, 0.82, 0.78), vec3(0.8, 0.66, 0.62), n * 0.5 + 0.5);
    float streaks = smoothstep(0.6, 0.9, gnoise(vec3(p.x * 40.0, p.y * 4.0, p.z * 40.0))) * cap;
    col = mix(mix(north, south, cap), vec3(0.3, 0.27, 0.27), streaks * 0.7);
  } else if (u_feature == 9) {  // Pluto: Tombaugh Regio 'heart', dark Cthulhu Macula
    col = mix(vec3(0.72, 0.6, 0.48), vec3(0.85, 0.75, 0.62), smoothstep(-0.3, 0.3, n));
    float lonDeg = degrees(lon);
    if (lonDeg < 0.0) lonDeg += 360.0;
    float belt = smoothstep(26.0, 12.0, abs(degrees(lat) + 2.0)) * smoothstep(10.0, 30.0, lonDeg)
               * smoothstep(165.0, 140.0, lonDeg);
    col = mix(col, vec3(0.3, 0.16, 0.11), clamp(belt + n2 * 0.3, 0.0, 1.0) * belt);
    float west = smoothstep(0.38, 0.3, angle(p, dirAt(22.0, 177.0)) + n2 * 0.05);
    float east = smoothstep(0.5, 0.36, angle(p, dirAt(2.0, 215.0)) + n2 * 0.08);
    col = mix(col, vec3(0.97, 0.94, 0.9), max(west, east * 0.85));
    col = mix(col, vec3(0.75, 0.7, 0.62), smoothstep(1.1, 1.4, lat) * 0.6);
  } else if (u_feature == 10) {  // Charon: grey ice, red north pole (Mordor Macula)
    col = mix(col, vec3(0.42, 0.22, 0.15), smoothstep(1.0, 1.3, lat + n * 0.15));
  } else if (u_feature == 11) {  // Ceres: dark rock, bright salt in Occator crater
    float spot = smoothstep(0.035, 0.015, angle(p, dirAt(19.8, 239.3)))
               + smoothstep(0.03, 0.012, angle(p, dirAt(20.4, 241.0))) * 0.7;
    col = mix(col, vec3(0.97, 0.96, 0.92), clamp(spot, 0.0, 1.0));
  } else if (u_feature == 12) {  // icy dwarf planets: smooth, mottled
    col = mix(u_color2, u_color, smoothstep(-0.4, 0.4, n + 0.5 * n2));
  } else if (u_feature == 14) {  // spacecraft: painted white dish, booms and bus
    col = u_color;
  } else if (u_feature == 13) {  // Miranda: patchwork with grooved coronae
    float region = cellDist(p * 1.3 + u_seed);
    float stripes = 0.5 + 0.5 * sin(region * 60.0);
    col = mix(col, mix(vec3(0.42), vec3(0.8), stripes), smoothstep(0.55, 0.45, region));
  }
  return col * (1.0 + h * 0.8);
}

float occlusion(vec3 p, vec3 L, float dSun, vec4 occ) {
  vec3 oc = occ.xyz - p;
  float t = dot(oc, L);
  if (t <= 0.0 || t > dSun) return 1.0;
  float d = length(oc - L * t);
  float pen = u_sunRadius * t / dSun;
  return smoothstep(occ.w - pen, occ.w + pen, d);
}

void main() {
  gl_FragDepth = log2(v_logz) * u_logF;
  vec3 V = normalize(-v_world);
  vec3 N = normalize(u_basis * normalize(v_local / u_scale));
  if (u_bumpy > 0.0) {
    vec3 dn = normalize(cross(dFdx(v_world), dFdy(v_world)));
    if (dot(dn, V) < 0.0) dn = -dn;
    N = dn;
  }

  if (u_shade == 0) {  // the Sun: emissive, darker at the limb
    vec3 c = u_hasTex == 1 ? texture(u_tex0, v_uv).rgb : u_color;
    float mu = max(dot(N, V), 0.0);
    float limb = 0.4 + 0.6 * pow(mu, 0.5);
    float boil = gnoise(normalize(v_local) * 40.0 + u_time * 0.15) * 0.1;
    o_color = vec4(c * vec3(1.45, 1.2, 0.9) * limb * (1.0 + boil) + vec3(0.3, 0.12, 0.0) * (1.0 - mu), 1.0);
    return;
  }

  if (u_shade == 4) {  // a hypothetical planet: a faint outline
    float f = pow(1.0 - abs(dot(N, V)), 2.0);
    o_color = vec4(u_color * (0.08 + 0.9 * f), 0.08 + 0.55 * f);
    return;
  }

  vec3 toSun = u_sun - v_world;
  float dSun = length(toSun);
  vec3 L = toSun / dSun;

  vec3 albedo;
  if (u_hasTex == 1) {
    albedo = texture(u_tex0, v_uv).rgb;
  } else {
    vec3 p = normalize(v_local);
    float h = height(p);
    albedo = surface(p, h);
    // Bump the normal from the height field.
    vec3 t1 = normalize(cross(abs(p.y) < 0.99 ? vec3(0, 1, 0) : vec3(1, 0, 0), p));
    vec3 t2 = cross(p, t1);
    float e = 0.004;
    float h1 = height(normalize(p + t1 * e));
    float h2 = height(normalize(p + t2 * e));
    float relief = u_feature == 7 ? 0.3 : 0.04 + 0.26 * clamp(u_craters, 0.0, 1.0);
    vec3 bumped = normalize(p - (t1 * (h1 - h) + t2 * (h2 - h)) / e * relief);
    N = normalize(N + u_basis * (bumped - p));
  }

  float shadow = 1.0;
  for (int k = 0; k < 4; k++) {
    if (k >= u_occN) break;
    shadow *= occlusion(v_world, L, dSun, u_occ[k]);
  }
  if (u_ring.z > 0.0) {  // the rings' shadow
    float denom = dot(L, u_ringNormal);
    if (abs(denom) > 1e-4) {
      float s = dot(u_center - v_world, u_ringNormal) / denom;
      if (s > 0.0) {
        float r = length(v_world + L * s - u_center);
        if (r > u_ring.x && r < u_ring.y) {
          float t = (r - u_ring.x) / (u_ring.y - u_ring.x);
          float a = u_ring.w > 0.5 ? texture(u_ringTex, vec2(t, 0.5)).a : 0.5;
          shadow *= 1.0 - a * u_ring.z * 0.8;
        }
      }
    }
  }

  float ndl = dot(N, L);
  float mu = max(dot(N, V), 0.0);
  vec3 col;
  if (u_shade == 3) {  // Earth
    float cloud = texture(u_tex2, v_uv + vec2(u_time * 0.0015, 0.0)).r;
    float lit = max(ndl, 0.0) * shadow;
    vec3 day = mix(albedo, vec3(1.0), cloud * 0.92);
    float dark = smoothstep(0.12, -0.12, ndl);
    vec3 night = texture(u_tex1, v_uv).rgb * vec3(1.0, 0.82, 0.55) * 1.6 * dark * (1.0 - cloud * 0.85);
    float water = smoothstep(0.03, 0.12, albedo.b - albedo.r) * (1.0 - cloud);
    float spec = pow(max(dot(reflect(-L, N), V), 0.0), 40.0) * 0.45 * water * lit;
    col = day * (lit * 1.1 + 0.012) + night + vec3(1.0, 0.95, 0.85) * spec;
  } else if (u_shade == 2) {  // gas and cloud: soft terminator, limb darkening
    float lit = pow(max(ndl, 0.0), 0.75) * shadow;
    col = albedo * (lit * 1.15 + 0.012) * mix(0.62, 1.0, pow(mu, 0.35));
  } else {  // rock and ice
    float lit = max(ndl, 0.0) * shadow;
    col = albedo * (lit * 1.2 + 0.018);
  }

  if (u_atmo.a > 0.0) {
    float rim = pow(1.0 - mu, 2.5) * u_atmo.a * smoothstep(-0.25, 0.45, ndl);
    col += u_atmo.rgb * rim * 0.7;
  }
  o_color = vec4(col, 1.0);
}
