#version 300 es

precision highp float;
precision highp usampler2D;

uniform usampler2D u_tex;
uniform float u_max_iter;
uniform float u_offset_x;
uniform float u_offset_y;

in vec2 v_uv;
out vec4 outColor;

vec3 colormap(float t) {
    return vec3(0.5 + 0.5*sin(6.2831 * (t + vec3(0.0, 0.33, 0.67))));
}

void main() {
    vec2 uv = v_uv + vec2(u_offset_x, u_offset_y);
    uint val = texture(u_tex, fract(uv)).r;
    float t = float(val) / u_max_iter;
    outColor = vec4(colormap(t), 1.0);
}
