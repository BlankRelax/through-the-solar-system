in vec3 v_color;
in float v_shape;
in float v_logz;

uniform float u_logF;

out vec4 o_color;

void main() {
  gl_FragDepth = log2(v_logz) * u_logF;
  vec2 d = gl_PointCoord - 0.5;
  float r = length(d);
  float a;
  if (v_shape < 0.5) {
    a = smoothstep(0.5, 0.25, r);
  } else if (v_shape < 1.5) {  // spacecraft: a diamond outline with a bright centre
    float m = abs(d.x) + abs(d.y);
    a = smoothstep(0.08, 0.0, abs(m - 0.4)) + smoothstep(0.14, 0.04, r);
  } else if (v_shape < 2.5) {  // hypothetical: a dashed circle
    float dash = step(0.0, sin(atan(d.y, d.x) * 8.0));
    a = smoothstep(0.05, 0.0, abs(r - 0.42)) * dash;
  } else {  // selection: four corner brackets
    vec2 q = abs(d);
    a = smoothstep(0.03, 0.0, abs(max(q.x, q.y) - 0.46)) * step(0.3, min(q.x, q.y));
  }
  if (a <= 0.01) discard;
  a = min(a, 1.0);
  o_color = vec4(v_color * a, a);
}
