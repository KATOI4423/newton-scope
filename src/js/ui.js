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
const uploadBtn = document.getElementById('uploadBtn');

const spinner = document.getElementById('spinner');

let prevFormula = "";
let prevMaxIter;

const invoke = window.__TAURI__.core.invoke;
const { confirm, message } = window.__TAURI__.dialog;

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
    maxIter.value = await invoke("get_default_max_iter");
    iterRange.value = maxIter.value;
    prevMaxIter = maxIter.value;
    updateIterRangeBackground();
    await setCenterStr();
    await setScaleStr();

    setSize();
    // setMaxIter(maxIter.value);
    // plot();
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
    const f = fexpr.value;

    const delayMsec = 500;
    let showSpinner = true;
    const timer = setTimeout(() => {
        if (showSpinner) spinner.style.display = "flex";
    }, delayMsec);

    try {
        const ret = await invoke("set_formula", {formula: f});
        if (ret !== "OK") {
            showSpinner = false; // エラー時はスピナー表示を止める
            await message(ret, {title: "Failed to set f(z)", kind: "error"});
            fexpr.value = prevFormula;
            throw Error("Failed to set formula:", f, ret); // 例外を送信することで、エラーログに出力 & finallyの終了処理を確実に行う
        }
        // plot();
    } catch (e) {
        console.error(e);
    } finally {
        showSpinner = false;
        clearTimeout(timer);
        spinner.style.display = "none";
        prevFormula = f;
    }
}

fexpr.addEventListener('change', setFexpr);

// link iter inputs
iterRange.addEventListener('input', async () => {
    let value = iterRange.valueAsNumber;
    maxIter.value = value;
    await innerSetMaxIter(value);
});
maxIter.addEventListener('change', async () => {
    let value = maxIter.valueAsNumber;
    if (!maxIter.validity.valid) {
        await message(
            `Out of range: ${maxIter.min} ~ ${maxIter.max}`,
            { title: "Failed to set Max iterations", kind: "error" }
        );
        maxIter.value = prevMaxIter;
        return;
    }
    iterRange.value = value;
    await innerSetMaxIter(value);
});

async function innerSetMaxIter(value) {
    try {
        await invoke("set_max_iter", { maxIter: value });
        updateMaxIter(value);
        updateTile();
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
canvas.addEventListener('mousemove', async (e) => {
    if (!isDragging)
        return;

    const rect = canvas.getBoundingClientRect();
    const [canvasX, canvasY] = getCanvasCoordinate(e);
    const [dx, dy] = [canvasX - lastX, canvasY - lastY];
    [lastX, lastY] = [canvasX, canvasY];

    await invoke("move_view", { dx: dx/rect.width, dy: dy/rect.height });
    await setCenterStr();
    await updateTile();
});
canvas.addEventListener('wheel', async (e) => {
    e.preventDefault();

    const zoomLevel = (e.deltaY > 0) ? 1 : -1; // 奥がズームイン,手前がズームアウト

    await invoke("zoom_view", { level: zoomLevel });
    await setScaleStr();
    await updateTile();
});
