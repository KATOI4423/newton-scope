// shader.js

import { setCenterStr } from "./ui.js";

const invoke = window.__TAURI__.core.invoke;

let gl;
let program;
let uniforms = {};
let texture;

/**
 * WebGL2初期化関数
 * @param {string} canvasId canvas要素のID
 * @param {string} vertexPath vertex.glslのパス
 * @param {string} fragmentPath fragment.glslのパス
 * @returns {void}
 */
async function initGL(canvasId, vertexPath, fragmentPath) {
    const canvas = document.getElementById(canvasId);
    gl = canvas.getContext("webgl2");
    if (!gl) {
        throw new Error("WebGL2 not supported");
    }
    
    const [vsSource, fsSource] = await Promise.all([
        fetch(vertexPath).then(r => r.text()),
        fetch(fragmentPath).then(r => r.text())
    ]);

    const vs = compileShader(gl.VERTEX_SHADER, vsSource);
    const fs = compileShader(gl.FRAGMENT_SHADER, fsSource);
    program = linkProgram(vs, fs);
    gl.useProgram(program);

    gl.viewport(0, 0, gl.canvas.width, gl.canvas.height);
    gl.clearColor(0, 0, 0, 1);
}

/**
 * Shaderをコンパイル
 * @param {GLenum} type gl.VERTEX_SHADER または gl.FRAGMENT_SHADER
 * @param {string} src Shaderソースコード
 * @returns {WebGLShader} コンパイル済みシェーダオブジェクト
 */
function compileShader(type, src) {
    const shader = gl.createShader(type);
    gl.shaderSource(shader, src);
    gl.compileShader(shader);
    if (!gl.getShaderParameter(shader, gl.COMPILE_STATUS)) {
        console.error(gl.getShaderInfoLog(shader));
        throw new Error("Shader compile failed");
    }
    return shader;
}

/**
 * shaderとfragmentをprogramにセットし、リンクする
 * @param {WebGLShader} vs 頂点シェーダ
 * @param {WebGLShader} fs フラグメントシェーダ
 * @returns {WebGLProgram} リンク済みプログラム
 */
function linkProgram(vs, fs) {
    const program = gl.createProgram();
    gl.attachShader(program, vs);
    gl.attachShader(program, fs);
    gl.linkProgram(program);
    if (!gl.getProgramParameter(program, gl.LINK_STATUS)) {
        console.error(gl.getProgramInfoLog(program));
        throw new Error("Program link frailed");
    }
    return program;
}

/**
 * フルスクリーン矩形のVAOを作成
 * @returns {void}
 */
function createQuad() {
    const quad = new Float32Array([
        -1, -1,  1, -1, -1,  1,
         1, -1,  1,  1, -1,  1,
    ]);

    const vao = gl.createVertexArray();
    gl.bindVertexArray(vao);

    const vbo = gl.createBuffer();
    gl.bindBuffer(gl.ARRAY_BUFFER, vbo);
    gl.bufferData(gl.ARRAY_BUFFER, quad, gl.STATIC_DRAW);

    const loc = gl.getAttribLocation(program, "a_pos");
    gl.enableVertexAttribArray(loc);
    gl.vertexAttribPointer(loc, 2, gl.FLOAT, false, 0, 0);
}

/**
 * Uint16ArrayからR16UIテクスチャを生成
 * @param {Uint16Array} data 16bit符号なし整数配列データ
 * @param {number} size テクスチャ1辺のサイズ
 * @returns {WebGLTexture} 生成されたテクスチャ
 */
function createTextureFromData(data, size) {
    const tex = gl.createTexture();
    gl.activeTexture(gl.TEXTURE0);
    gl.bindTexture(gl.TEXTURE_2D, tex);

    uniforms.u_tex = gl.getUniformLocation(program, "u_tex");
    gl.uniform1i(uniforms.u_tex, 0);

    gl.texImage2D(
        gl.TEXTURE_2D, 0, gl.R16UI, size, size, 0,
        gl.RED_INTEGER, gl.UNSIGNED_SHORT, data
    );
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.REPEAT);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.REPEAT);

    return tex;
}

/**
 * 最大反復回数をシェーダに更新
 * @param {number} maxIter 最大反復回数
 * @returns {void}
 */
export function updateMaxIter(maxIter) {
    uniforms.u_max_iter = gl.getUniformLocation(program, "u_max_iter");
    gl.uniform1f(uniforms.u_max_iter, maxIter);
}

/**
 * GPUテクスチャに部分更新をかける
 * @returns {void}
 */
export async function updateTile() {
    const size = document.getElementById('presetSize');
    const sizeValue = (size.value !== "") ?
        Number(size.value) : await invoke("get_default_size");
    const maxIter = document.getElementById('maxIter');
    const maxIterValue = (maxIter.value !== "") ?
        Number(maxIter.value) : await invoke("get_default_max_iter");

    const data = await invoke("generate_test_data", {
        tileSize: sizeValue,
        maxIter: maxIterValue,
    });
    const array = new Uint16Array(data);

    gl.bindTexture(gl.TEXTURE_2D, texture);
    gl.texSubImage2D(
        gl.TEXTURE_2D,
        0,      // level
        0, 0,   // offset-x, offset-y
        sizeValue,  // width
        sizeValue,  // height
        gl.RED_INTEGER,
        gl.UNSIGNED_SHORT,
        array
    );
}

/**
 * 毎フレームの描画更新
 * @returns {void}
 */
function renderFrame() {
    gl.clear(gl.COLOR_BUFFER_BIT);
    gl.drawArrays(gl.TRIANGLES, 0, 6);
    requestAnimationFrame(renderFrame);
}


/**
 * 初期化エントリポイント
 * @returns {Promise<void>}
 */
async function main() {
    const texSize = await invoke("get_default_size");
    const maxIter = await invoke("get_default_max_iter");

    await initGL("fractal", "/glsl/vertex.glsl", "/glsl/fragment.glsl");
    createQuad();

    const testData = await invoke("generate_test_data", {
        tileSize: texSize,
        maxIter: maxIter,
    });
    const array = new Uint16Array(testData);
    texture = createTextureFromData(array, texSize);
    updateMaxIter(maxIter);

    renderFrame();
    await updateTile();
}

window.addEventListener("DOMContentLoaded", main);
