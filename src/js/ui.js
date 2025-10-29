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
const renderBtn = document.getElementById('renderBtn');
const resetBtn = document.getElementById('resetBtn');
const saveBtn = document.getElementById('saveBtn');
const uploadBtn = document.getElementById('uploadBtn');

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
    size.value = await invoke("get_size");
    maxIter.value = await invoke("get_max_iter");
    iterRange.value = maxIter.value;
    center.textContent = await invoke("get_center_str");
    scale.textContent = await invoke("get_scale_str");
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

    renderBtn.click();
}

fexpr.addEventListener('change', setFexpr);

// link iter inputs
iterRange.addEventListener('input', () => { maxIter.value = iterRange.value; });
maxIter.addEventListener('change', () => { iterRange.value = maxIter.value; });

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
