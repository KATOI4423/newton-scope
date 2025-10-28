// shader.js

async function loadShaderSource(url) {
    const res = await fetch(url);
    if (!res.ok) {
        throw new Error(`Failed to load shader: ${url}`);
    }
    return await res.text();
}

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

async function createProgram(gl) {
    const [vertexShaderSource, fragmentShaderSource] = await Promise.all([
        loadShaderSource("/glsl/vertex.glsl"),
        loadShaderSource("/glsl/fragment.glsl")
    ]);

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

async function setup() {
    const canvas = document.getElementById("fractal");
    const gl = canvas.getContext("webgl2");
    if (!gl) {
        alert("WebGL2 not supported");
        return;
    }

    const program = await createProgram(gl);
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
