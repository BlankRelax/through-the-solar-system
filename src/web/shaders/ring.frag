// Ring brightness from a texture strip (Saturn) or procedural bands, lit
// from whichever side faces the Sun, with the planet's shadow across it.

in vec2 v_xz;
in vec3 v_world;
in float v_logz;

uniform vec2 u_radii;
uniform int u_hasTex;
uniform sampler2D u_ringTex;
uniform vec3 u_color;
uniform float u_opacity;
uniform vec3 u_sun;
uniform vec3 u_center;
uniform float u_planetR;
uniform vec3 u_normal;
uniform float u_logF;

out vec4 o_color;

void main() {
  gl_FragDepth = log2(v_logz) * u_logF;
  float t = (length(v_xz) - u_radii.x) / (u_radii.y - u_radii.x);
  if (t < 0.0 || t > 1.0) discard;
  vec4 c;
  if (u_hasTex == 1) {
    c = texture(u_ringTex, vec2(t, 0.5));
  } else {
    float bands = 0.55 + 0.45 * sin(t * 70.0 + sin(t * 13.0) * 3.0);
    c = vec4(u_color, bands * smoothstep(0.0, 0.08, t) * smoothstep(1.0, 0.92, t));
  }
  vec3 L = normalize(u_sun - v_world);
  float sunSide = dot(L, u_normal);
  float viewSide = dot(-v_world, u_normal);
  float lit = 0.2 + 0.8 * sqrt(abs(sunSide));
  if (sunSide * viewSide < 0.0) lit *= 0.5 * (1.0 - c.a * 0.6);  // the unlit face
  vec3 oc = u_center - v_world;
  float s = dot(oc, L);
  if (s > 0.0) lit *= mix(0.05, 1.0, smoothstep(u_planetR * 0.97, u_planetR * 1.02, length(oc - L * s)));
  o_color = vec4(c.rgb * lit * 1.15, c.a * u_opacity);
}
