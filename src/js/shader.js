// shader.js

const invoke = window.__TAURI__.core.invoke;

let gl;
let program;
let uniforms = {};
const texture = {
    front: null,
    back:  null,
};
const fbo = {
    read: null,
    draw: null,
};
const shiftOffset = {
    x:  0.0,
    y:  0.0,
    location: null,
};

/**
 * WebGL2初期化関数
 * @param {string} canvasId canvas要素のID
 * @param {string} vertexPath vertex.glslのパス
 * @param {string} fragmentPath fragment.glslのパス
 * @returns {void}
 */
async function initGL(canvasId, vertexPath, fragmentPath) {
    const canvas = document.getElementById(canvasId);
    gl = canvas.getContext("webgl2", {
        antialias: false,
        depth: false,
        preserveDrawingBuffer: true
    });
    if (!gl) {
        throw new Error("WebGL2 not supported");
    }

    if (fbo.read) gl.deleteFramebuffer(fbo.read);
    if (fbo.draw) gl.deleteFramebuffer(fbo.draw);
    fbo.read = gl.createFramebuffer();
    fbo.draw = gl.createFramebuffer();

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

/** テクスチャ作成ヘルパー */
function createTextureObject() {
    gl.activeTexture(gl.TEXTURE0);
    const tex = gl.createTexture();
    if (!tex) throw new Error("Failed to create texture object");

    gl.bindTexture(gl.TEXTURE_2D, tex);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MIN_FILTER, gl.NEAREST);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_MAG_FILTER, gl.NEAREST);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_S, gl.CLAMP_TO_EDGE);
    gl.texParameteri(gl.TEXTURE_2D, gl.TEXTURE_WRAP_T, gl.CLAMP_TO_EDGE);
    gl.bindTexture(gl.TEXTURE_2D, null);
    return tex;
}

function initTexture() {
    if (!gl) return;

    if (texture.front) gl.deleteTexture(texture.front);
    if (texture.back) gl.deleteTexture(texture.back);
    texture.front = createTextureObject();
    texture.back = createTextureObject();

    gl.useProgram(program);
    gl.activeTexture(gl.TEXTURE0);
    gl.bindTexture(gl.TEXTURE_2D, texture.front);

    uniforms.u_tex = gl.getUniformLocation(program, "u_tex");
    gl.uniform1i(uniforms.u_tex, 0);

    // 描写毎にu_shift_offsetの値が変わるので、値の設定はrenderFrameに任せる
    shiftOffset.location = gl.getUniformLocation(program, "u_shift_offset");

    // Uint16Array (2Byte) のデータ転送時、幅が奇数だとアライメントズレが発生するため、
    // パッキングのアライメントを 1Byte に設定して防ぐ
    gl.pixelStorei(gl.UNPACK_ALIGNMENT, 1);
}

/**
 * テクスチャのメモリ領域をsizeに応じて確保する
 * データ転送は行わず、VRAMの確保のみを行う
 * @param {number} size
 */
export function resizeTexture(size) {
    if (!gl) return;

    if (!texture.front || !texture.back) {
        console.warn("Textures not initialized, initializing now...");
        initTexture();
    }

    if (!texture.front || !texture.back) {
        throw new Error("Texture initialization failed insize resizeTextrue");
    }

    gl.activeTexture(gl.TEXTURE0);

    gl.bindTexture(gl.TEXTURE_2D, texture.front);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.R16UI, size, size, 0, gl.RED_INTEGER, gl.UNSIGNED_SHORT, null);

    gl.bindTexture(gl.TEXTURE_2D, texture.back);
    gl.texImage2D(gl.TEXTURE_2D, 0, gl.R16UI, size, size, 0, gl.RED_INTEGER, gl.UNSIGNED_SHORT, null);

    gl.bindTexture(gl.TEXTURE_2D, texture.front);
}

/**
 * テクスチャにデータをアップロードする
 * @param {Uint16Array} data 更新するピクセルデータ
 * @param {number} width データの幅 (x方向)
 * @param {number} height データの高さ (y方向)
 * @param {number} x 更新開始位置 (default: 0)
 * @param {number} y 更新開始位置 (default: 0)
 */
export function updateTexture(data, width, height, x = 0, y = 0) {
    if (!texture.front || !gl) return;

    gl.bindTexture(gl.TEXTURE_2D, texture.front);
    gl.texSubImage2D(
        gl.TEXTURE_2D,
        0,      // level
        x, y,   // offset
        width, height,
        gl.RED_INTEGER,
        gl.UNSIGNED_SHORT,
        data
    );
}

/**
 * 最大反復回数をシェーダに更新
 * @param {number} maxIter 最大反復回数
 * @returns {void}
 */
export function updateMaxIter(maxIter) {
    if (!gl) return;
    uniforms.u_max_iter = gl.getUniformLocation(program, "u_max_iter");
    gl.uniform1f(uniforms.u_max_iter, maxIter);
}

/**
 * ビューポートサイズを更新する
 * @param {number} size 新しいcanvasサイズ
 */
export function updateViewport(size) {
    if (!gl) return;
    gl.viewport(0, 0, size, size);
}

/**
 * 移動によって新規描写する領域を計算する
 * @param {number} dx 移動したピクセル量
 * @param {number} dy 移動したピクセル量
 * @param {number} size canvasのサイズ
 */
function calc_request_rects(dx, dy, size) {
    const requests = [];

    if (dx !== 0) {
        // dx > 0 の場合、左側に空白ができる
        if (dx > 0) {
            requests.push({ x: 0, y: 0, w: dx, h: size });
        }
        // dx < 0 の場合、右側に空白ができる
        else {
            const absDx = Math.abs(dx);
            requests.push({ x: size - absDx, y: 0, w: absDx, h: size });
        }
    }

    if (dy !== 0) {
        // dy > 0 の場合、下側に空白ができる
        if (dy > 0) {
            requests.push({ x: 0, y: size - dy, w: size, h: dy });
        }
        // dy < 0 の場合、上川に空白ができる
        else {
            const absDy = Math.abs(dy);
            requests.push({ x: 0, y: 0, w: size, h: absDy });
        }
    }

    return requests;
}

/**
 * FrameBufferObjectを使用して、 texture.front から texture.back へずらしてコピーする (Blit)
 * @param {number} size canvasのサイズ
 * @param {number} pixelDx 移動ピクセル数
 * @param {number} pixelDy 移動ピクセル数
 */
function slideCopyTexture(size, pixelDx, pixelDy) {
    gl.bindFramebuffer(gl.READ_FRAMEBUFFER, fbo.read);
    gl.framebufferTexture2D(gl.READ_FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, texture.front, 0);

    gl.bindFramebuffer(gl.DRAW_FRAMEBUFFER, fbo.draw);
    gl.framebufferTexture2D(gl.DRAW_FRAMEBUFFER, gl.COLOR_ATTACHMENT0, gl.TEXTURE_2D, texture.back, 0);

    const statusRead = gl.checkFramebufferStatus(gl.READ_FRAMEBUFFER);
    const statusDraw = gl.checkFramebufferStatus(gl.DRAW_FRAMEBUFFER);
    if (statusRead !== gl.FRAMEBUFFER_COMPLETE || statusDraw !== gl.FRAMEBUFFER_COMPLETE) {
        console.error("Framebuffer incomplete", statusRead, statusDraw);
        return;
    }

    let srcX0 = 0, srcY0 = 0, srcX1 = size, srcY1 = size;
    let dstX0 = 0, dstY0 = 0, dstX1 = size, dstY1 = size;

    if (pixelDx > 0) {
        srcX1 = size - pixelDx;
        dstX0 = pixelDx;
    } else {
        srcX0 = -pixelDx;
        dstX1 = size + pixelDx;
    }

    // ブラウザのy軸は下方向が正だが、canvasのy軸は上方向が正のため、処理を逆転させる
    if (pixelDy > 0) {
        srcY0 = pixelDy;
        dstY1 = size - pixelDy;
    } else {
        srcY1 = size + pixelDy;
        dstY0 = -pixelDy;
    }

    gl.blitFramebuffer(
        srcX0, srcY0, srcX1, srcY1,
        dstX0, dstY0, dstX1, dstY1,
        gl.COLOR_BUFFER_BIT, gl.NEAREST // 整数テクスチャのため NEAREST 必須
    );

    gl.bindFramebuffer(gl.READ_FRAMEBUFFER, null);
    gl.bindFramebuffer(gl.DRAW_FRAMEBUFFER, null);
}

/** texture.front と texture.back を入れ替える (Ping-Pong) */
function switchTexture() {
    let temp = texture.front;
    texture.front = texture.back;
    texture.back = temp;
}

/**
 * GPUテクスチャに部分更新をかける
 * @param {number} pixelDx X軸方向の移動量
 * @param {number} pixelDy Y軸方向の移動量
 * @returns {void}
 */
export async function updateTile(pixelDx, pixelDy) {
    if (!gl) return;
    const totalSize = await invoke("get_size");

    if ((pixelDx === 0) && (pixelDy === 0)) {
        renderFrame();
        return;
    }

    // 移動量が画面サイズ以上の場合はBlitできないので、全画面書き換えとする
    if (Math.abs(pixelDx) >= totalSize || Math.abs(pixelDy) >= totalSize) {
        const data = await invoke("render_tile", { x: 0, y: 0, w: totalSize, h: totalSize });
        const array = new Uint16Array(data);
        updateTexture(array, totalSize, totalSize, 0, 0);
        renderFrame();
        return;
    }

    slideCopyTexture(totalSize, pixelDx, pixelDy);
    switchTexture();

    gl.activeTexture(gl.TEXTURE0);
    gl.bindTexture(gl.TEXTURE_2D, texture.front);

    shiftOffset.x = 0.0; shiftOffset.y = 0.0;

    // Blitによりデータ自体が移動済みなので、ここですぐに renderFrame すると
    // 空白部分は「前のフレームの残りカス」が表示されるが、一瞬なので許容する
    // 気になる場合は、ここで renderFrame を呼ばない
    renderFrame();

    const requests = calc_request_rects(pixelDx, pixelDy, totalSize);
    if (requests.length === 0) {
        renderFrame();
        return;
    }

    const updatePromises = requests.map(async (req) => {
        const data = await invoke("render_tile", {
            x: req.x, y: req.y, w: req.w, h: req.h
        });

        const array = new Uint16Array(data);
        updateTexture(array, req.w, req.h, req.x, req.y);
    });

    await Promise.all(updatePromises);

    renderFrame();
}

/**
 * 毎フレームの描画更新
 * @returns {void}
 */
function renderFrame() {
    if (!gl) return;
    gl.clear(gl.COLOR_BUFFER_BIT);
    if (shiftOffset.location) {
        gl.uniform2f(shiftOffset.location, shiftOffset.x, shiftOffset.y);
    }
    gl.drawArrays(gl.TRIANGLES, 0, 6);
}

function startRenderLoop() {
    function loop() {
        renderFrame();
        requestAnimationFrame(loop);
    }
    loop();
}

/**
 * 初期化エントリポイント
 * @returns {Promise<void>}
 */
export async function initGLSetup() {
    const texSize = await invoke("get_default_size");
    const maxIter = await invoke("get_default_max_iter");

    await initGL("fractal", "/glsl/vertex.glsl", "/glsl/fragment.glsl");
    createQuad();

    initTexture();
    resizeTexture(texSize);
    updateMaxIter(maxIter);

    await updateTile(0, 0);
    startRenderLoop();
}
