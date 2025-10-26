// shader.js

// 頂点シェーダ
const vertexShaderSource = `#version 300 es
in vec2 a_position;
out vec2 v_position;
void main() {
    v_position = a_position;
    gl_Position = vec4(a_position, 0.0, 1.0);
}
`;

// フラグメントシェーダ
const fragmentShaderSource = `#version 300 es
precision highp float;

in vec2 v_position;
out vec4 outColor;

// Uniforms (CPUから設定)
uniform float u_scale;      // スケール S
uniform int u_order;        // Taylor展開の次数
uniform vec2 u_coeffs[8];   // 最大8項分の係数（実部, 虚部）
uniform int u_repmax;       // ニュートン法の反復最大値

// 複素数を vec2 として扱う
vec2 cadd(vec2 a, vec2 b) { return a + b; }
vec2 cmul(vec2 a, vec2 b) { return vec2(a.x*b.x - a.y*b.y, a.x*b.y + a.y*b.x); }
vec2 cdiv(vec2 a, vec2 b) {
    float coef = 1.0 / (b.x*b.x + b.y*b.y);
    float re = a.x*b.x + a.y*b.y;
    float im = a.y*b.x - a.x*b.y;
    return vec2(re, im) * coef;
}

vec4 jet(float t) {
    t = clamp(t, 0.0, 1.0);
    float r = clamp(1.5 - abs(4.0 * t - 3.0), 0.0, 1.0);
    float g = clamp(1.5 - abs(4.0 * t - 2.0), 0.0, 1.0);
    float b = clamp(1.5 - abs(4.0 * t - 1.0), 0.0, 1.0);
    return vec4(r, g, b, 1.0);
}

// Taylor展開でのfとdfを、Horner法により求める
void eval_f_and_df(vec2 z, out vec2 f, out vec2 df) {
    vec2 sz = u_scale * z; // s * z

    f = u_coeffs[u_order];
    df = u_coeffs[u_order + 1];

    for (int i = u_order - 1; i >= 0; --i) {
        vec2 sz_inv_i = sz / float(i + 1);
        f = cadd(cmul(f, sz_inv_i), u_coeffs[i]);
        df = cadd(cmul(df, sz_inv_i), u_coeffs[i + 1]);
    }
}

// ニュートン法により、z_{n+1}を計算する
vec2 newton_method(vec2 z) {
    vec2 f = vec2(0.0, 0.0);
    vec2 df = vec2(0.0, 0.0);

    // Taylor展開でのf & dfを、Horner法により求める
    eval_f_and_df(z, f, df);

    return z - cdiv(f, u_scale * df);
}

bool is_converged(vec2 z1, vec2 z2) {
    const float EPSILON = 1.0e-5;
    return all(lessThan(abs(z1 - z2), vec2(EPSILON)));
}

void main() {
    vec2 z1 = v_position;
    vec2 z2 = z1;
    float iter = float(u_repmax);

    for (int i = 0; i < u_repmax; ++i) {
        z2 = newton_method(z1);
        if (is_converged(z1, z2)) {
            iter = float(i);
            break;
        }
        z1 = z2;
    }

    outColor = jet(iter / float(u_repmax));
    // outColor = jet(log(iter + 1.0) / log(float(u_repmax) + 1.0));
}
`;

function createShader(gl, type, source) {
    const shader = gl.createShader(type);
    gl.shaderSource(shader, source);
    gl.compileShader(shader);
    if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
        console.error(gl.getShaderInfoLog(shader));
        gl.deleteShader(shader);
        return null;
    }
    return shader;
}

function createProgram(gl) {
    const vs = createShader(gl, gl.VERTEX_SHADER, vertexShaderSource);
    const fs = createShader(gl, gl.FRAGMENT_SHADER, fragmentShaderSource);
    const program = gl.createProgram();
    gl.attachShader(program, vs);
    gl.attachShader(program, fs);
    gl.linkProgram(program);
    if (!gl.getProgramParameter(program, gl.LINK_STATUS)) {
        console.error(gl.getProgramInfoLog(program));
        return null;
    }
    return program;
}

function setup() {
    const canvas = document.getElementById("fractal");
    const gl = canvas.getContext("webgl2");
    if (!gl) {
        alert("WebGL2 not supported");
        return;
    }

    const program = createProgram(gl);
    gl.useProgram(program);

    // 頂点データ(全画面矩形)
    const positions = new Float32Array([
        -1, -1,  1, -1, -1,  1,
        -1,  1,  1, -1,  1,  1
    ]);
    const vao = gl.createVertexArray();
    gl.bindVertexArray(vao);
    const posBuffer = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, posBuffer);
    gl.bufferData(gl.ARRAY_BUFFER, positions, gl.STATIC_DRAW);
    const posLoc = gl.getAttribLocation(program, "a_position");
    gl.enableVertexAttribArray(posLoc);
    gl.vertexAttribPointer(posLoc, 2, gl.FLOAT, false, 0, 0);

    // Uniform locations
    const u_scale  = gl.getUniformLocation(program, "u_scale");
    const u_order  = gl.getUniformLocation(program, "u_order");
    const u_coeffs = gl.getUniformLocation(program, "u_coeffs");
    const u_repmax = gl.getUniformLocation(program, "u_repmax");

    // パラメータ設定（例）
    gl.uniform1f(u_scale, 2.0);
    gl.uniform1i(u_order, 3);
    gl.uniform1i(u_repmax, 512);

    // 係数（`f(z) = z^3 - 1` のTaylor展開）
    const coeffs = new Float32Array([
        -1.0, 0.0,      // a0
        0.0, 0.0,       // a1
        0.0, 0.0,       // a2
        1.0, 0.0,       // a3 (z^3)
        0.0, 0.0,       // 残りは空
        0.0, 0.0,
        0.0, 0.0,
        0.0, 0.0,
    ]);
    gl.uniform2fv(u_coeffs, coeffs);

    // 描画
    gl.viewport(0, 0, canvas.width, canvas.height);
    gl.clearColor(0, 0, 0, 1);
    gl.clear(gl.COLOR_BUFFER_BIT);
    gl.drawArrays(gl.TRIANGLES, 0, 6);
}

window.addEventListener("DOMContentLoaded", setup);
