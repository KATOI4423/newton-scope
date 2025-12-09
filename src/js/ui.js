// ui.js

import {
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


let prevFormula = "";
let prevMaxIter;

let wheelAccum = 0;

const invoke = window.__TAURI__.core.invoke;
const { confirm, message } = window.__TAURI__.dialog;


/**
 * 連続して呼ばれる処理を、一定間隔に1回実行する
 * @param {Function} func 実行する処理
 * @param {Number} interval 実行間隔[msec]
 */
function throttle(func, interval = 100) {
    let lastExecTime = 0;
    let lastArgs = null;
    let timerId = null;

    return (...args) => {
        const now = Date.now();
        lastArgs = args;

        if (now - lastExecTime >= interval) {
            func(...args);
            lastExecTime = now;
        } else {
            clearTimeout(timerId);
            timerId = setTimeout(() => {
                func(...lastArgs);
                lastExecTime = Date.now();
            }, interval);
        }
    };
}

/**
 * 一定時間経過しても処理が完了しない場合、スピナーを表示する
 * @param {Function} func 実行する処理
 * @param {Number} delayMsec スピナーを表示するまでの時間[msec]
 */
async function setSpinner(func, delayMsec = 500) {
    let showSpinner = true;
    const timer = setTimeout(() => {
        if (!showSpinner)
            return;

        elements.spinner.style.display = "flex";
        console.info("The operation is taking longer than expected...");
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

// default setting
async function setDefault(isUserClidked) {
    if (isUserClidked) {
        const ok = await confirm('Are you sure to reset all configures?',
            { title: '', type: 'warning' }
        );
        if (!ok) return;
    }

    await invoke("initialize");
    elements.fexpr.value = await invoke("get_default_formula");
    prevFormula = elements.fexpr.value;
    elements.presetSize.value = await invoke("get_default_size");
    const maxIterValue = await invoke("get_default_max_iter");
    syncMaxIterInputs(maxIterValue);
    updateMaxIter(maxIterValue);
    prevMaxIter = maxIterValue;
    await setCenterStr();
    await setScaleStr();
    wheelAccum = 0;

    await setSize();
    await setSpinner(updateTile);
}

async function initialize() {
    await setDefault(false);
}

async function clickedReset() {
    await setDefault(true);
}

elements.resetBtn.addEventListener('click', clickedReset);
window.addEventListener("DOMContentLoaded", initialize);

export async function setCenterStr() {
    elements.center.textContent = await invoke("get_center_str");
}

export async function setScaleStr() {
    elements.scale.textContent = await invoke("get_scale_str");
}

async function setFexpr() {
    await setSpinner(async () => {
        const f = elements.fexpr.value;
        const ret = await invoke("set_formula", { formula: f });
        if (ret !== "OK") {
            elements.fexpr.value = prevFormula;
            await message(ret, { title: "Failed to set f(z)", kind: "error" });
            throw Error("Failed to set formula", f, ret);
        }

        await updateTile();
        prevFormula = f;
    });
}

elements.fexpr.addEventListener('change', setFexpr);

const throttleSetMaxIter = throttle(async (value) => {
    await innerSetMaxIter(value);
}, 100);

elements.iterRange.addEventListener('input', async () => {
    let value = elements.iterRange.valueAsNumber;
    syncMaxIterInputs(value);
    throttleSetMaxIter(value);
});
elements.maxIter.addEventListener('change', async () => {
    let value = elements.maxIter.valueAsNumber;
    if (!elements.maxIter.validity.valid) {
        await message(
            `Out of range: ${elements.maxIter.min} ~ ${elements.maxIter.max}`,
            { title: "Failed to set Max iterations", kind: "error" }
        );
        syncMaxIterInputs(prevMaxIter);
        return;
    }

    syncMaxIterInputs(value);
    await innerSetMaxIter(value);
});

function syncMaxIterInputs(value) {
    elements.maxIter.value = value;
    elements.iterRange.value = value;
    updateIterRangeBackground();
}

async function innerSetMaxIter(value) {
    try {
        await invoke("set_max_iter", { maxIter: value });
        updateMaxIter(value);
        await setSpinner(updateTile);
        prevMaxIter = value;
    } catch (error) {
        await message(
            error,
            { title: "Failed to set Max iterations", kind: "error" }
        );
        elements.maxIter.value = prevMaxIter;
        elements.iterRange.value = prevMaxIter;
        updateIterRangeBackground();
    }
}

function updateIterRangeBackground() {
    const baseColor = '#ccc';
    const activeColor = '#ff4500';

    const progress = (elements.iterRange.value / elements.iterRange.max) * 100;
    elements.iterRange.style.background = `linear-gradient(to right, ${activeColor} ${progress}%, ${baseColor} ${progress}%)`;
}
elements.iterRange.addEventListener('input', updateIterRangeBackground);
elements.maxIter.addEventListener('change', updateIterRangeBackground);

async function setSize() {
    const value = Number(elements.presetSize.value);
    elements.canvas.width = value;
    elements.canvas.height = value;
    updateViewport(value);
    resizeTexture(value);
    await invoke("set_size", { size: value });
}

elements.presetSize.addEventListener("change", async () => {
    await setSize();
    await setSpinner(updateTile);
});

// fractalの描写要素を正方形に保つ
function adjustCanvasSize() {
    const wrapRect = elements.wrap.getBoundingClientRect();
    const coordsRect = elements.coords.getBoundingClientRect();

    const availableHeight = wrapRect.height - coordsRect.height;
    const availableWidth = wrapRect.width;

    const size = Math.min(availableWidth, availableHeight);
    elements.canvas.style.width = `${size}px`;
    elements.canvas.style.height = `${size}px`;
}

window.addEventListener("load", adjustCanvasSize);
window.addEventListener("resize", adjustCanvasSize);

let isDragging = false;
let lastX = 0;
let lastY = 0;

function getCanvasCoordinate(mouseEvent) {
    const rect = elements.canvas.getBoundingClientRect();
    return [mouseEvent.clientX - rect.left, mouseEvent.clientY - rect.top];
}

elements.canvas.addEventListener('mousedown', (e) => {
    isDragging = true;

    [lastX, lastY] = getCanvasCoordinate(e);
    console.debug(lastX, lastY);
});
elements.canvas.addEventListener('mouseup', () => {
    isDragging = false;
});
elements.canvas.addEventListener('mouseleave', () => {
    isDragging = false;
});

const throttleMoveView = throttle(async (e) => {
    const rect = elements.canvas.getBoundingClientRect();
    const [canvasX, canvasY] = getCanvasCoordinate(e);
    const [dx, dy] = [canvasX - lastX, canvasY - lastY];
    [lastX, lastY] = [canvasX, canvasY];

    await invoke("move_view", { dx: dx / rect.width, dy: dy / rect.height });
    await setCenterStr();
    await setSpinner(updateTile);
}, 30);

elements.canvas.addEventListener('mousemove', async (e) => {
    if (!isDragging)
        return;
    throttleMoveView(e);
});

const throttleZoomView = throttle(async (e) => {
    if (wheelAccum === 0)
        return;

    const level = (wheelAccum > 0) ? -1 : 1;
    wheelAccum = 0;

    const rect = elements.canvas.getBoundingClientRect();
    const [canvasX, canvasY] = getCanvasCoordinate(e);

    await invoke("zoom_view", { level, x: canvasX / rect.width, y: canvasY / rect.height });
    await setScaleStr();
    await setCenterStr();
    await setSpinner(updateTile);
}, 30);

elements.canvas.addEventListener('wheel', async (e) => {
    e.preventDefault();
    wheelAccum += e.deltaY;
    throttleZoomView(e);
});
