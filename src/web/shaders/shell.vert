// The heliosphere's boundaries, already in display space.

layout(location = 0) in vec3 a_pos;
layout(location = 1) in vec3 a_normal;

uniform mat4 u_viewProj;
uniform vec3 u_origin;

out vec3 v_world;
out vec3 v_normal;
out float v_logz;

void main() {
  v_world = u_origin + a_pos;
  v_normal = a_normal;
  gl_Position = u_viewProj * vec4(v_world, 1.0);
  v_logz = 1.0 + gl_Position.w;
}
