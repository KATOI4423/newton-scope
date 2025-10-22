// shader.js

// 頂点シェーダ
export const vertexShaderSource = `#version 300 es
in vec2 a_position;
out vec2 v_position;
void main() {
    v_position = a_position;
    gl_Position = vec4(a_position, 0.0, 1.0);
}
`;

// フラグメントシェーダ
export const fragmentShaderSource = `#version 300 es
precision highp float;

in vec2 v_position;
out vec4 outColor;

// Uniforms (CPUから設定)
uniform vec2 u_center;      // 展開中心 C
uniform float u_scale;      // スケール S
uniform int u_order;        // Taylor展開の次数
uniform vec2 u_coeffs[8];   // 最大8項分の係数（実部, 虚部）

// 複素数を vec2 として扱う
vec2 cadd(vec2 a, vec2 b) { return a + b; }
vec2 cmul(vec2 a, vec2 b) { return vec2(a.x*b.x - a.y*b.y, a.x*b.y + a.y*b.x); }
vec2 cpow(vec2 z, int n) {
    vec2 r = vec2(1.0, 0.0);
    for (int i=0; i<n; ++i) r = cmul(r, z);
    return r;
}

void main() {
    // 相対座標 z' = (z - C) / S
    vec2 z = v_position * u_scale + u_center;
    vec2 zrel = (z - u_center) / u_scale;

    // Taylor多項式 f(z) ~ sum{ a_k * (z')^k }
    vec2 f = vec2(0.0);
    for (int k=0; k<8; ++k) {
        if (k >= u_order) break;
        vec2 term = cmul(u_coeffs[k], cpow(zrel, k));
        f = cadd(f, term);
    }

    // 出力を色にマッピング（実部で赤、虚部で青）
    outColor = vec4(f.x*0.5 + 0.5, 0.0, f.y*0.5 + 0.5, 1.0);
}
`;
