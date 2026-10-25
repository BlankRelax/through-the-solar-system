in float v_logz;

uniform vec4 u_color;
uniform float u_logF;

out vec4 o_color;

void main() {
  gl_FragDepth = log2(v_logz) * u_logF;
  o_color = u_color;
}
