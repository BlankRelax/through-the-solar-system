// Dots for bodies too small to see, the spacecraft, and the selection mark.

layout(location = 0) in vec3 a_pos;     // relative to the camera
layout(location = 1) in vec4 a_style;   // colour, size in pixels
layout(location = 2) in float a_shape;  // 0 dot, 1 spacecraft, 2 hypothetical, 3 selection

uniform mat4 u_viewProj;

out vec3 v_color;
out float v_shape;
out float v_logz;

void main() {
  gl_Position = u_viewProj * vec4(a_pos, 1.0);
  v_logz = 1.0 + gl_Position.w;
  gl_PointSize = a_style.w;
  v_color = a_style.rgb;
  v_shape = a_shape;
}
