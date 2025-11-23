// ui.js

import {
    updateMaxIter,
    updateTile,
} from "./shader.js";

const wrap = document.querySelector('.canvas-wrap');
const canvas = document.getElementById('fractal');
const coords = document.querySelector('.coords');
const center = document.getElementById('center');
const scale = document.getElementById('scale');

const fexpr = document.getElementById('fexpr');
const size = document.getElementById('presetSize');
const maxIter = document.getElementById('maxIter');
const iterRange = document.getElementById('iterRange');
const resetBtn = document.getElementById('resetBtn');
const saveBtn = document.getElementById('saveBtn');
const importBtn = document.getElementById('importBtn');

const spinner = document.getElementById('spinner');

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

        spinner.style.display = "flex";
        console.info("The operation is taking longer than expected...");
    }, delayMsec);

    try {
        await func();
    } catch (e) {
        console.error(e);
    } finally {
        showSpinner = false;
        clearTimeout(timer);
        spinner.style.display = "none";
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
    fexpr.value = await invoke("get_default_formula");
    prevFormula = fexpr.value;
    size.value = await invoke("get_default_size");
    const maxIterValue = await invoke("get_default_max_iter");
    syncMaxIterInputs(maxIterValue);
    updateMaxIter(maxIterValue);
    prevMaxIter = maxIterValue;
    await setCenterStr();
    await setScaleStr();
    wheelAccum = 0;

    setSize();
    await updateTile();
}

async function initialize() {
    await setDefault(false);
}

async function clickedReset() {
    await setDefault(true);
}

resetBtn.addEventListener('click', clickedReset);
window.addEventListener("DOMContentLoaded", initialize);

export async function setCenterStr() {
    center.textContent = await invoke("get_center_str");
}

export async function setScaleStr() {
    scale.textContent = await invoke("get_scale_str");
}

async function setFexpr() {
    await setSpinner(async () => {
        const f = fexpr.value;
        const ret = await invoke("set_formula", { formula: f });
        if (ret !== "OK") {
            fexpr.value = prevFormula;
            await message(ret, { title: "Failed to set f(z)", kind: "error" });
            throw Error("Failed to set formula", f, ret);
        }

        await updateTile();
        prevFormula = f;
    });
}

fexpr.addEventListener('change', setFexpr);

const throttleSetMaxIter = throttle(async (value) => {
    await innerSetMaxIter(value);
}, 100);

iterRange.addEventListener('input', async () => {
    let value = iterRange.valueAsNumber;
    syncMaxIterInputs(value);
    throttleSetMaxIter(value);
});
maxIter.addEventListener('change', async () => {
    let value = maxIter.valueAsNumber;
    if (!maxIter.validity.valid) {
        await message(
            `Out of range: ${maxIter.min} ~ ${maxIter.max}`,
            { title: "Failed to set Max iterations", kind: "error" }
        );
        syncMaxIterInputs(prevMaxIter);
        return;
    }

    syncMaxIterInputs(value);
    await innerSetMaxIter(value);
});

function syncMaxIterInputs(value) {
    maxIter.value = value;
    iterRange.value = value;
    updateIterRangeBackground();
}

async function innerSetMaxIter(value) {
    try {
        await invoke("set_max_iter", { maxIter: value });
        updateMaxIter(value);
        await updateTile();
        prevMaxIter = value;
    } catch (error) {
        await message(
            error,
            { title: "Failed to set Max iterations", kind: "error" }
        );
        maxIter.value = prevMaxIter;
        iterRange.value = prevMaxIter;
        updateIterRangeBackground();
    }
}

function updateIterRangeBackground() {
    const baseColor = '#ccc';
    const activeColor = '#ff4500';

    const progress = (iterRange.value / iterRange.max) * 100;
    iterRange.style.background = `linear-gradient(to right, ${activeColor} ${progress}%, ${baseColor} ${progress}%)`;
}
iterRange.addEventListener('input', updateIterRangeBackground);
maxIter.addEventListener('change', updateIterRangeBackground);

function setSize() {
    const value = size.value;
    canvas.width = value;
    canvas.height = value;
    // setViewPort(value, value);
}

size.addEventListener("change", () => {
    setSize();
    // plot();
});

// fractalの描写要素を正方形に保つ
function adjustCanvasSize() {
    const wrapRect = wrap.getBoundingClientRect();
    const coordsRect = coords.getBoundingClientRect();

    const availableHeight = wrapRect.height - coordsRect.height;
    const availableWidth = wrapRect.width;

    const size = Math.min(availableWidth, availableHeight);
    canvas.style.width = `${size}px`;
    canvas.style.height = `${size}px`;
}

window.addEventListener("load", adjustCanvasSize);
window.addEventListener("resize", adjustCanvasSize);

let isDragging = false;
let lastX = 0;
let lastY = 0;

function getCanvasCoordinate(mouseEvent) {
    const rect = canvas.getBoundingClientRect();
    return [mouseEvent.clientX - rect.left, mouseEvent.clientY - rect.top];
}

canvas.addEventListener('mousedown', (e) => {
    isDragging = true;

    [lastX, lastY] = getCanvasCoordinate(e);
    console.debug(lastX, lastY);
});
canvas.addEventListener('mouseup', () => {
    isDragging = false;
});
canvas.addEventListener('mouseleave', () => {
    isDragging = false;
});

const throttleMoveView = throttle(async (e) => {
    const rect = canvas.getBoundingClientRect();
    const [canvasX, canvasY] = getCanvasCoordinate(e);
    const [dx, dy] = [canvasX - lastX, canvasY - lastY];
    [lastX, lastY] = [canvasX, canvasY];

    await invoke("move_view", { dx: dx / rect.width, dy: dy / rect.height });
    await setCenterStr();
    await updateTile();
}, 30);

canvas.addEventListener('mousemove', async (e) => {
    if (!isDragging)
        return;
    throttleMoveView(e);
});

const throttleZoomView = throttle(async (e) => {
    if (wheelAccum === 0)
        return;

    const level = (wheelAccum > 0) ? -1 : 1;
    wheelAccum = 0;

    const rect = canvas.getBoundingClientRect();
    const [canvasX, canvasY] = getCanvasCoordinate(e);

    await invoke("zoom_view", { level, x: canvasX / rect.width, y: canvasY / rect.height });
    await setScaleStr();
    await setCenterStr();
    await updateTile();
}, 30);

canvas.addEventListener('wheel', async (e) => {
    e.preventDefault();
    wheelAccum += e.deltaY;
    throttleZoomView(e);
});
