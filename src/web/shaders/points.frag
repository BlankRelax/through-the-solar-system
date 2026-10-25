in vec3 v_color;
in float v_alpha;
in float v_logz;

uniform float u_logF;

out vec4 o_color;

void main() {
  gl_FragDepth = log2(v_logz) * u_logF;
  float r = length(gl_PointCoord - 0.5);
  if (r > 0.5) discard;
  float a = smoothstep(0.5, 0.1, r) * v_alpha;
  o_color = vec4(v_color * a, a);
}
