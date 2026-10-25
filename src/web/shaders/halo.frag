// The glow of an atmosphere seen edge-on, just beyond a planet's limb.
// Drawn on the back faces of a slightly larger sphere, added to the scene.

in vec3 v_local;
in vec3 v_world;
in vec2 v_uv;
in float v_logz;

uniform vec3 u_center;
uniform float u_radius;
uniform float u_thick;
uniform vec4 u_atmo;
uniform vec3 u_sun;
uniform float u_logF;

out vec4 o_color;

void main() {
  gl_FragDepth = log2(v_logz) * u_logF;
  vec3 dir = normalize(v_world);
  float t = dot(u_center, dir);
  vec3 cp = dir * t - u_center;  // centre to the ray's closest approach
  float b = length(cp);
  float x = (b - u_radius) / u_thick;
  if (t < 0.0 || x < 0.0) discard;
  float density = exp(-x * 4.5) * (1.0 - smoothstep(0.75, 1.0, x));
  float light = smoothstep(-0.35, 0.3, dot(cp / b, normalize(u_sun - u_center)));
  o_color = vec4(u_atmo.rgb * density * light * u_atmo.a, 1.0);
}
