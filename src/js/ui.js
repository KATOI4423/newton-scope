// ui.js

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

const invoke = window.__TAURI__.core.invoke;
const { confirm, message } = window.__TAURI__.dialog;

import {
    plot,
    setCoeffs,
    setMaxIter,
    setViewPort,
} from "./shader.js"

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
    size.value = await invoke("get_default_size");
    maxIter.value = await invoke("get_default_max_iter");
    iterRange.value = maxIter.value;
    updateIterRangeBackground();
    center.textContent = await invoke("get_center_str");
    scale.textContent = await invoke("get_scale_str");

    setSize();
    setMaxIter(maxIter.value);
    await setCoeffs();
    plot();
}

async function initialize() {
    await setDefault(false);
}

async function clickedReset() {
    await setDefault(true);
}

resetBtn.addEventListener('click', clickedReset);
window.addEventListener("DOMContentLoaded", initialize);

async function setFexpr() {
    const f = fexpr.value;
    const ret = await invoke("set_formula", {formula: f});
    if (ret !== "OK") {
        console.error("Failed to set formula:", f, ret);
        await message(ret, {title: "Failed to set f(z)", kind: "error"});
        return;
    }
    console.log("Successed to set formula", f);

    await setCoeffs();
    plot();
}

fexpr.addEventListener('change', setFexpr);

// link iter inputs
iterRange.addEventListener('input', () => {
    maxIter.value = iterRange.value;
    setMaxIter(iterRange.value);
    plot();
});
maxIter.addEventListener('change', () => {
    iterRange.value = maxIter.value;
    setMaxIter(maxIter.value);
    plot();
});

function updateIterRangeBackground() {
    const baseColor = '#ccc';
    const activeColor = '#ff4500';

    const progress = (iterRange.value / iterRange.max) * 100;
    iterRange.style.background = `linear-gradient(to right, ${activeColor} ${progress}%, ${baseColor} ${progress}%)`;
}
iterRange.addEventListener('input', updateIterRangeBackground);

function setSize() {
    const value = size.value;
    canvas.width = value;
    canvas.height = value;
    setViewPort(value, value);
}

size.addEventListener("change", () => {
    setSize();
    plot();
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
