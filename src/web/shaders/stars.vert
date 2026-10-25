// Background stars, infinitely far away.

layout(location = 0) in vec3 a_dir;
layout(location = 1) in float a_bright;
layout(location = 2) in vec3 a_color;

uniform mat4 u_viewProj;
uniform float u_pointScale;

out vec3 v_color;

void main() {
  vec4 p = u_viewProj * vec4(a_dir, 0.0);
  p.z = p.w * 0.99999;
  gl_Position = p;
  gl_PointSize = (1.0 + a_bright * 2.4) * u_pointScale;
  v_color = a_color * (0.3 + a_bright * 0.9);
}
