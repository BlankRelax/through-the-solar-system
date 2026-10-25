// Orbit paths, relative to the body they go around.

layout(location = 0) in vec3 a_pos;

uniform mat4 u_viewProj;
uniform vec3 u_origin;

out float v_logz;

void main() {
  gl_Position = u_viewProj * vec4(u_origin + a_pos, 1.0);
  v_logz = 1.0 + gl_Position.w;
}
