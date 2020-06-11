import { BinaryCell } from "progenitor";
import { memory } from "progenitor/progenitor_bg";

const c = BinaryCell.new();
console.log('c.answer():', c.answer());

/*
const CELL_SIZE = 5; // px
const GRID_COLOR = "#CCCCCC";
const DEAD_COLOR = "#FFFFFF";
const ALIVE_COLOR = "#000000";

const u = Universe.new();
const width = u.get_width()
const height = u.get_height()

const canvas = document.getElementById('canvas');
canvas.height = (CELL_SIZE + 1) * height + 1;
canvas.width = (CELL_SIZE + 1) * width + 1;
const ctx = canvas.getContext('2d');

const renderLoop = () => {
    // canvas.textContent = u.render()
    for (let i=0; i<3; i++)
        u.tick();

    drawGrid();
    drawCells();

    requestAnimationFrame(renderLoop)
}

requestAnimationFrame(renderLoop)

const drawGrid = () => {
    ctx.beginPath();
    ctx.strokeColor = GRID_COLOR;

    // Vertical lines.
    for (let i = 0; i <= width; i++) {
        ctx.moveTo(i * (CELL_SIZE + 1) + 1, 0);
        ctx.lineTo(i * (CELL_SIZE + 1) + 1, (CELL_SIZE + 1) * height + 1);
    }

    // Horizontal lines.
    for (let j = 0; j <= height; j++) {
        ctx.moveTo(0,                           j * (CELL_SIZE + 1) + 1);
        ctx.lineTo((CELL_SIZE + 1) * width + 1, j * (CELL_SIZE + 1) + 1);
    }

    ctx.stroke();
}

const getIndex = (row, column) => {
    return row * width + column;
};

const drawCells = () => {
    let cells = new Uint8Array(memory.buffer, u.cells(), width * height);

    ctx.beginPath();
    for (let row = 0; row < height; row++) {
        for (let col = 0; col < width; col++) {
        const idx = getIndex(row, col);

        ctx.fillStyle = cells[idx] === Cell.Dead
            ? DEAD_COLOR
            : ALIVE_COLOR;

        ctx.fillRect(
            col * (CELL_SIZE + 1) + 1,
            row * (CELL_SIZE + 1) + 1,
            CELL_SIZE,
            CELL_SIZE
        );
        }
    }
    ctx.stroke();
}

*/
