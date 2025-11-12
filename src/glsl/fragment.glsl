#version 300 es

precision highp float;
precision highp usampler2D;

uniform usampler2D u_tex;
uniform float u_max_iter;

in vec2 v_uv;
out vec4 outColor;

vec3 jet(float t) {
    t = clamp(t, 0.0, 1.0);
    float r = clamp(1.5 - abs(4.0 * t - 3.0), 0.0, 1.0);
    float g = clamp(1.5 - abs(4.0 * t - 2.0), 0.0, 1.0);
    float b = clamp(1.5 - abs(4.0 * t - 1.0), 0.0, 1.0);
    return vec3(r, g, b);
}

void main() {
    uint val = texture(u_tex, v_uv).r;
    float t = float(val) / u_max_iter;
    outColor = vec4(jet(t), 1.0);
}
