// shader.js

const invoke = window.__TAURI__.core.invoke;
let gl;
let u_coeffs;
let u_max_iter;

async function loadShaderSource(url) {
    const res = await fetch(url);
    if (!res.ok) {
        throw new Error(`Failed to load shader: ${url}`);
    }
    return await res.text();
}

function createShader(type, source) {
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

async function createProgram() {
    const [vertexShaderSource, fragmentShaderSource] = await Promise.all([
        loadShaderSource("/glsl/vertex.glsl"),
        loadShaderSource("/glsl/fragment.glsl")
    ]);

    const vs = createShader(gl.VERTEX_SHADER, vertexShaderSource);
    const fs = createShader(gl.FRAGMENT_SHADER, fragmentShaderSource);
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
    gl = canvas.getContext("webgl2");
    if (!gl) {
        alert("WebGL2 not supported");
        return;
    }

    const program = await createProgram();
    if (!program) {
        console.error("Failed to create Program");
        return;
    }
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
    setViewPort(canvas.width, canvas.height)

    u_max_iter = gl.getUniformLocation(program, "u_max_iter");
    u_coeffs = gl.getUniformLocation(program, "u_coeffs");
}

window.addEventListener("DOMContentLoaded", setup);

export function plot() {
    gl.clearColor(0, 0, 0, 1);
    gl.clear(gl.COLOR_BUFFER_BIT);
    gl.drawArrays(gl.TRIANGLES, 0, 6);
}

export async function setCoeffs() {
    const coeffs = new Float32Array(await invoke("get_coeffs"));
    gl.uniform2fv(u_coeffs, coeffs);
}

export function setMaxIter(maxIter) {
    gl.uniform1i(u_max_iter, maxIter);
}

export function setViewPort(w, h) {
    gl.viewport(0, 0, w, h);
}