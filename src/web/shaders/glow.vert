// A camera-facing quad: the Sun's glare, a comet's coma and tail.

uniform mat4 u_proj;
uniform vec3 u_centerView;
uniform vec3 u_axisX;
uniform vec3 u_axisY;

out vec2 v_uv;
out float v_logz;

void main() {
  vec2 c = vec2(float(gl_VertexID & 1), float(gl_VertexID >> 1)) * 2.0 - 1.0;
  v_uv = c;
  gl_Position = u_proj * vec4(u_centerView + u_axisX * c.x + u_axisY * c.y, 1.0);
  v_logz = 1.0 + gl_Position.w;
}
