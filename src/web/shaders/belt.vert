// Asteroids, Kuiper belt objects and Oort cloud comets: each point is a
// circular orbit moved here, then squeezed to the game's distance scale.

layout(location = 0) in vec4 a_orbit;   // a [AU], inclination, node, phase [rad]
layout(location = 1) in vec2 a_motion;  // mean motion [rad/day], size
layout(location = 2) in vec3 a_color;

uniform mat4 u_viewProj;
uniform vec3 u_cam;
uniform float u_days;
uniform float u_scale;      // display units per sqrt(AU)
uniform float u_pxPerUnit;  // pixels per display unit at distance 1
uniform float u_size;
uniform float u_alpha;

out vec3 v_color;
out float v_alpha;
out float v_logz;

void main() {
  float u = a_orbit.w + a_motion.x * u_days;
  float ci = cos(a_orbit.y), si = sin(a_orbit.y);
  float cn = cos(a_orbit.z), sn = sin(a_orbit.z);
  float cu = cos(u), su = sin(u);
  vec3 ecl = vec3(cn * cu - sn * su * ci, sn * cu + cn * su * ci, su * si);
  vec3 world = vec3(ecl.x, ecl.z, -ecl.y) * (u_scale * sqrt(a_orbit.x));
  vec3 rel = world - u_cam;
  gl_Position = u_viewProj * vec4(rel, 1.0);
  v_logz = 1.0 + gl_Position.w;
  float px = a_motion.y * u_size * u_pxPerUnit / max(length(rel), 1e-4);
  gl_PointSize = clamp(px, 1.0, 4.0);
  v_alpha = u_alpha * clamp(px * 1.6, 0.3, 1.0);
  v_color = a_color;
}
