// Planetary rings: a flat annulus in the planet's equatorial plane.

layout(location = 0) in vec3 a_ring;  // cos, sin, edge (0 inner, 1 outer)

uniform mat4 u_viewProj;
uniform vec3 u_center;
uniform mat3 u_basis;
uniform float u_radius;
uniform vec2 u_radii;  // in planet radii

out vec2 v_xz;
out vec3 v_world;
out float v_logz;

void main() {
  float r = mix(u_radii.x, u_radii.y, a_ring.z) * 1.004;
  vec3 local = vec3(a_ring.x * r, 0.0, -a_ring.y * r);
  v_xz = local.xz;
  v_world = u_center + u_basis * (local * u_radius);
  gl_Position = u_viewProj * vec4(v_world, 1.0);
  v_logz = 1.0 + gl_Position.w;
}
