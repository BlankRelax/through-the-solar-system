// Spheres, ellipsoids and lumpy small bodies. Everything is positioned
// relative to the camera, so the GPU never sees large coordinates.

layout(location = 0) in vec3 a_pos;
layout(location = 1) in vec2 a_uv;

uniform mat4 u_viewProj;
uniform vec3 u_center;  // body centre relative to the camera
uniform mat3 u_basis;   // prime meridian, north pole, and their cross product
uniform vec3 u_scale;   // semi-axes in display units
uniform vec3 u_offset;  // lobe offset (contact binaries), display units
uniform float u_bumpy;
uniform float u_seed;

out vec3 v_local;
out vec3 v_world;
out vec2 v_uv;
out float v_logz;

void main() {
  vec3 p = a_pos;
  if (u_bumpy > 0.0) p *= 1.0 + u_bumpy * fbm(a_pos * 1.7 + u_seed);
  v_local = p;
  v_uv = a_uv;
  v_world = u_center + u_basis * (p * u_scale + u_offset);
  gl_Position = u_viewProj * vec4(v_world, 1.0);
  v_logz = 1.0 + gl_Position.w;
}
