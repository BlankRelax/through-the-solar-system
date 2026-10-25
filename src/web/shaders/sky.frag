// The Milky Way, sampled in galactic coordinates so it lies where it
// really does on the sky.

in vec2 v_ndc;

uniform mat4 u_invViewProj;
uniform mat3 u_toGalactic;
uniform sampler2D u_tex;
uniform float u_intensity;

out vec4 o_color;

void main() {
  vec4 w = u_invViewProj * vec4(v_ndc, 1.0, 1.0);
  vec3 g = u_toGalactic * normalize(w.xyz / w.w);
  float l = atan(g.y, g.x);
  float b = asin(clamp(g.z, -1.0, 1.0));
  vec2 uv = vec2(0.5 - l / 6.2831853, 0.5 - b / 3.14159265);
  vec3 c = textureLod(u_tex, uv, 0.0).rgb;
  o_color = vec4(c * u_intensity, 1.0);
}
