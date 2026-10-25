// Brightest where the shell is seen edge-on, like a soap bubble.

in vec3 v_world;
in vec3 v_normal;
in float v_logz;

uniform vec3 u_color;
uniform float u_alpha;
uniform float u_logF;

out vec4 o_color;

void main() {
  gl_FragDepth = log2(v_logz) * u_logF;
  float f = pow(1.0 - abs(dot(normalize(v_normal), normalize(v_world))), 3.0);
  o_color = vec4(u_color * f * u_alpha, 1.0);
}
