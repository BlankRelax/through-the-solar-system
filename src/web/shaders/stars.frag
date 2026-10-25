in vec3 v_color;

out vec4 o_color;

void main() {
  float a = smoothstep(0.5, 0.0, length(gl_PointCoord - 0.5));
  o_color = vec4(v_color * a, 1.0);
}
