// shader.js

const invoke = window.__TAURI__.core.invoke;

let gl;
let program;
let uniforms = {};
let texture;
let offsetX = 0.0;
let offsetY = 0.0;


/**
 * WebGL2初期化関数
 * @param {string} canvasId canvas要素のID
 * @param {string} vertexPath vertex.glslのパス
 * @param {string} fragmentPath fragment.glslのパス
 * @returns {WebGL2RenderingContext} WebGL2コンテキスト
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

    return gl;
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
function updateMaxIter(maxIter) {
    uniforms.u_max_iter = gl.getUniformLocation(program, "u_max_iter");
    gl.uniform1f(uniforms.u_max_iter, maxIter);
}

/**
 * 水平方向のオフセットを更新
 * @param {number} offsetX 水平方向のオフセット
 * @returns {void}
 */
function updateOffsetX(offsetX) {
    uniforms.u_offset_x = gl.getUniformLocation(program, "u_offset_x");
    gl.uniform1f(uniforms.u_offset_x, offsetX);
}

/**
 * 垂直方向のオフセットを更新
 * @param {number} offsetY 垂直方向のオフセット
 * @returns {void}
 */
function updateOffsetY(offsetY) {
    uniforms.u_offset_y = gl.getUniformLocation(program, "u_offset_y");
    gl.uniform1f(uniforms.u_offset_y, offsetY);
}

/**
 * 毎フレームの描画更新
 * @returns {void}
 */
function renderFrame() {
    gl.clear(gl.COLOR_BUFFER_BIT);
    gl.uniform1f(uniforms.u_offset_x, offsetX);
    gl.uniform1f(uniforms.u_offset_y, offsetY);
    gl.drawArrays(gl.TRIANGLES, 0, 6);
    requestAnimationFrame(renderFrame);
}

/**
 * 入力処理をセットアップ
 * @returns {void}
 */
function setupInputHandlers() {
    const step = 0.01;

    window.addEventListener("keydown", (e) => {
        if (e.key === "ArrowRight") offsetX += step;
        if (e.key === "ArrowLeft")  offsetX -= step;
        if (e.key === "ArrowUp")    offsetY += step;
        if (e.key === "ArrowDown")  offsetY -= step;
    });
}

/**
 * テストデータ生成 // TODO: Rustでの計算結果に置き換える
 * @param {number} size 配列およびテクスチャ1辺のサイズ
 * @param {number} maxIter 最大反復回数
 * @returns {Uint16Array} 生成されたグレースケールデータ
 */
function generateTestData(size, maxIter) {
    const data = new Uint16Array(size * size);

    for (let y = 0; y < size; ++y) {
        const ys = y * size;
        const dy = y / size;
        for (let x = 0; x < size; ++x) {
            data[ys + x] = Math.floor(
                (0.5 + 0.5 * Math.sin((x / size) * 10.0 + dy * 6.0)) * maxIter
            );
        }
    }

    return data;
}

/**
 * 初期化エントリポイント
 * @returns {Promise<void>}
 */
async function main() {
    const texSize = 512;
    const maxIter = 4096.0;

    await initGL("fractal", "/glsl/vertex.glsl", "/glsl/fragment.glsl");
    createQuad();

    const testData = generateTestData(texSize, maxIter);
    texture = createTextureFromData(testData, texSize);
    updateMaxIter(maxIter);
    updateOffsetX(offsetX);
    updateOffsetY(offsetY);

    setupInputHandlers();
    renderFrame();
}

window.addEventListener("DOMContentLoaded", main);
