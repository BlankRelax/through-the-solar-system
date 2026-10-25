in vec2 v_uv;
in float v_logz;

uniform int u_mode;  // 0 the Sun, 1 comet tail, 2 comet coma
uniform vec3 u_color;
uniform float u_strength;
uniform float u_logF;

out vec4 o_color;

void main() {
  gl_FragDepth = log2(v_logz) * u_logF;
  float a;
  if (u_mode == 0) {
    float d = length(v_uv);
    a = exp(-d * 6.0) * 0.9 + 0.04 / (d * d * 30.0 + 0.04) * 0.6;
    float rays = pow(max(0.0, 1.0 - abs(v_uv.x * v_uv.y) * 60.0), 8.0) * exp(-d * 3.0) * 0.35;
    a = (a + rays) * smoothstep(1.0, 0.7, d);
  } else if (u_mode == 1) {
    float s = v_uv.x * 0.5 + 0.5;  // 0 at the nucleus, 1 at the end
    float across = v_uv.y / mix(0.12, 1.0, sqrt(s));
    a = pow(1.0 - s, 1.4) * exp(-across * across * 3.0) * smoothstep(0.0, 0.05, s);
  } else {
    float d = length(v_uv);
    a = exp(-d * 4.0) * smoothstep(1.0, 0.6, d);
  }
  o_color = vec4(u_color * a * u_strength, 1.0);
}
