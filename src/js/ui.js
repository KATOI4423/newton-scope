// ui.js

import {
    initGLSetup,
    resizeTexture,
    updateMaxIter,
    updateViewport,
    updateTile,
} from "./shader.js";

/* DOM Elements */
const elements = {
    wrap:       document.querySelector('.canvas-wrap'),
    canvas:     document.getElementById('fractal'),
    coords:     document.querySelector('.coords'),
    center:     document.getElementById('center'),
    scale:      document.getElementById('scale'),
    fexpr:      document.getElementById('fexpr'),
    presetSize: document.getElementById('presetSize'),
    maxIter:    document.getElementById('maxIter'),
    iterRange:  document.getElementById('iterRange'),
    resetBtn:   document.getElementById('resetBtn'),
    saveBtn:    document.getElementById('saveBtn'),
    importBtn:  document.getElementById('importBtn'),
    spinner:    document.getElementById('spinner'),
};

/* State Management */
const state = {
    prevFormula:    "",
    prevMaxIter:    0,
    wheelAccum:     0,
    isDragging:     false,
    // 入力蓄積用バッファ
    pendingMove:    { dx: 0, dy: 0 },
    pendingZoom:    { level: 0, x: 0, y: 0 },
    // 描画領域のキャッシュ
    rect:           { width: 1, height: 1, left: 0, top: 0 },
    // IPC制御フラグ
    isRendering:    false,
};

const invoke = window.__TAURI__.core.invoke;
const { confirm, message } = window.__TAURI__.dialog;


// ----- Utilities -----------------------------

/**
 * 一定時間経過しても処理が完了しない場合、スピナーを表示する
 * @param {Function} func 実行する処理
 * @param {Number} delayMsec スピナーを表示するまでの時間[msec]
 */
async function withSpinner(func, delayMsec = 500) {
    let showSpinner = true;
    const timer = setTimeout(() => {
        if (showSpinner) {
            elements.spinner.style.display = "flex";
        }
    }, delayMsec);

    try {
        await func();
    } catch (e) {
        console.error(e);
    } finally {
        showSpinner = false;
        clearTimeout(timer);
        elements.spinner.style.display = "none";
    }
}


// ----- Initialization & Config ---------------

/**
 * 設定値を初期化する
 * Resetボタンによる初期化の場合は、確認ダイアログを表示する
 * @param {Boolean} isUserClidked Resetボタンによる初期化かどうか
 */
async function setDefault(isUserClidked) {
    if (isUserClidked) {
        if(!await confirm('Are you sure to reset all configures?', { title: '', type: 'warning' }))
        {
            return;
        }
    }

    await invoke("initialize");

    const [defaultFormula, defaultSize, defaultMaxIter] = await Promise.all([
        invoke("get_default_formula"),
        invoke("get_default_size"),
        invoke("get_default_max_iter")
    ]);

    elements.fexpr.value = defaultFormula;
    state.prevFormula = defaultFormula;

    elements.presetSize.value = defaultSize;

    syncMaxIterInputs(defaultMaxIter);
    updateMaxIter(defaultMaxIter);
    state.prevMaxIter = defaultMaxIter;

    await Promise.all([updateInfoStr(), setSize()]);
    await withSpinner(updateTile);
}

/** GLSLと設定値を初期化 */
async function initialize() {
    await initGLSetup();
    await setDefault(false);
}


// ----- Update Logic --------------------------

/** CenterとScaleの情報を更新する */
async function updateInfoStr() {
    const [center, scale] = await Promise.all([
        invoke("get_center_str"),
        invoke("get_scale_str")
    ]);
    elements.center.textContent = center;
    elements.scale.textContent = scale;
}


// ----- Interaction Handlers ------------------

/** ResizeObserver: サイズ変更を検知し、rectをキャッシュする */
const resizeObserver = new ResizeObserver(() => {
    const wrapRect = elements.wrap.getBoundingClientRect();
    const coordsRect = elements.coords.getBoundingClientRect();

    const availableHeight = wrapRect.height - coordsRect.height;
    const availableWidth = wrapRect.width;
    const sizeVal = Math.min(availableWidth, availableHeight);

    elements.canvas.style.width = `${sizeVal}px`;
    elements.canvas.style.height = `${sizeVal}px`;

    // canvasのgetBoundingClientRectを呼ぶ回数を減らすため、値をグローバル変数にキャッシュする
    const r = elements.canvas.getBoundingClientRect();
    state.rect = {
        width: r.width,
        height: r.height,
        left: r.left,
        top: r.top,
    };
});
resizeObserver.observe(elements.wrap);

/**
 * Input Loop: requestAnimationFrameを使用して、GPUの準備ができている時だけ処理を行う
 * かつ、Rust側がBusyの場合は入力を蓄積する
 */
async function interactionLoop() {
    if (state.isRendering) {
        requestAnimationFrame(interactionLoop);
        return;
    }

    let needsUpdate = false;

    if (state.pendingZoom.level !== 0) {
        state.isRendering = true;
        const { level, x, y } = state.pendingZoom;

        // バッファをリセット
        state.pendingZoom = { level: 0, x: 0, y: 0 };

        try {
            await invoke("zoom_view", { level, x, y });
            await updateInfoStr();
            await updateTile();
        } finally {
            state.isRendering = false;
            // 処理中に入力があれば、次のループにて処理する
            requestAnimationFrame(interactionLoop);
        }
        return;
    }

    if ((state.pendingMove.dx !== 0) || (state.pendingMove.dy !== 0)) {
        state.isRendering = true;
        const normalizedDx = state.pendingMove.dx / state.rect.width;
        const normalizedDy = state.pendingMove.dy / state.rect.height;

        // バッファをリセット
        state.pendingMove = { dx: 0, dy: 0 };

        try {
            await invoke("move_view", {
                dx: normalizedDx,
                dy: normalizedDy
            });
            await updateInfoStr();
            await updateTile(normalizedDx, normalizedDy);
        } finally {
            state.isRendering = false;
            // 処理中に入力があれば、次のループにて処理する
            requestAnimationFrame(interactionLoop);
        }
        return;
    }

    requestAnimationFrame(interactionLoop);
}
// ループ開始
requestAnimationFrame(interactionLoop);


// ----- Event Listeners -----------------------

elements.canvas.addEventListener('pointerdown', (e) => {
    elements.canvas.setPointerCapture(e.pointerId);
    state.isDragging = true;
});
elements.canvas.addEventListener('pointerup', (e) => {
    elements.canvas.releasePointerCapture(e.pointerId);
    state.isDragging = false;
});
elements.canvas.addEventListener('pointermove', (e) => {
    if (!state.isDragging) {
        return;
    }

    state.pendingMove.dx += e.movementX;
    state.pendingMove.dy += e.movementY;
})

elements.canvas.addEventListener('wheel', (e) => {
    e.preventDefault();

    // e.clientX/Y はViewport基準なので、rect.left/topを引けばCanvas内座標になる
    const canvasX = e.clientX - state.rect.left;
    const canvasY = e.clientY - state.rect.top;

    const normX = canvasX / state.rect.width;
    const normY = canvasY / state.rect.height;

    const level = (e.deltaY > 0) ? -1 : 1;

    // 蓄積ではなく、最新のターゲットを更新する
    state.pendingZoom = { level, x: normX, y: normY };
}, { passive: false });


// ----- UI Component Handlers -----------------

async function setFexpr() {
    await withSpinner(async () => {
        const f = elements.fexpr.value;
        const ret = await invoke("set_formula", { formula: f });
        if (ret !== "OK") {
            elements.fexpr.value = state.prevFormula;
            await message(ret, { title: "Failed to set f(z)", kind: "error" });
            throw Error("Failed to set formula", f, ret);
        }

        await updateTile();
        state.prevFormula = f;
    });
}
elements.fexpr.addEventListener('change', setFexpr);

function syncMaxIterInputs(value) {
    elements.maxIter.value = value;
    elements.iterRange.value = value;

    const baseColor = '#ccc';
    const activeColor = '#ff4500';

    const progress = (elements.iterRange.value / elements.iterRange.max) * 100;
    elements.iterRange.style.background = `linear-gradient(to right, ${activeColor} ${progress}%, ${baseColor} ${progress}%)`;
}


// iterRangeはinputイベントが大量に来るので、Debounce処理を行う
let iterTimer;
elements.iterRange.addEventListener('input', () => {
    const value = elements.iterRange.valueAsNumber;
    syncMaxIterInputs(value);

    clearTimeout(iterTimer);
    iterTimer = setTimeout(async () => {
        await innerSetMaxIter(value);
    }, 100);
});

elements.maxIter.addEventListener('change', async () => {
    const value = elements.maxIter.valueAsNumber;
    if (!elements.maxIter.validity.valid) {
        await message("Out of range", { title: "Error", kind: "error" });
        syncMaxIterInputs(state.prevMaxIter);
        return;
    }
    syncMaxIterInputs(value);
    await innerSetMaxIter(value);
})

async function innerSetMaxIter(value) {
    try {
        await invoke("set_max_iter", { maxIter: value });
        updateMaxIter(value);
        await withSpinner(updateTile);
        state.prevMaxIter = value;
    } catch (error) {
        await message(error, { title: "Error", kind: "error" });
        syncMaxIterInputs(state.prevMaxIter);
    }
}

async function setSize() {
    const value = Number(elements.presetSize.value);
    elements.canvas.width = value;
    elements.canvas.height = value;

    updateViewport(value);
    resizeTexture(value);

    await invoke("set_size", { size: value });
}
elements.presetSize.addEventListener('change', async () => {
    await setSize();
    await withSpinner(updateTile);
});

elements.resetBtn.addEventListener('click', () => {
    setDefault(true);
});

window.addEventListener("DOMContentLoaded", initialize);
