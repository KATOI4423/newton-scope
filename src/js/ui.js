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
