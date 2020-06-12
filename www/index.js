import { BinaryCell } from "progenitor";
import { memory } from "progenitor/progenitor_bg";

import { SVG } from '@svgdotjs/svg.js'
import { defineGrid, extendHex } from 'honeycomb-grid'

const c = BinaryCell.new();
console.log('c.answer():', c.answer());

const svg = SVG().addTo('body').size(300, 300)

const Hex = extendHex({
  size: 13,
  render(svg) {
    const position = this.toPoint()
    const centerPosition = this.center().add(position)

    svg
      .polygon(this.corners().map(({ x, y }) => `${x},${y}`))
      .fill('none')
      .stroke({ width: 1, color: '#999' })
      .translate(position.x, position.y)

    /*
    svg
      .text(`${this.x},${this.y}`)
      .font({ size: 12, anchor: 'middle', leading: 1.4, fill: '#69c' })
      .translate(centerPosition.x, centerPosition.y - fontSize)
    */
  }
})
const Grid = defineGrid(Hex)
// Grid.parallelogram({ width: 10, height: 10, start: [0, 0], onCreate: renderHex})
Grid.rectangle({ width: 10, height: 10, onCreate: renderHex })

function renderHex(hex) {
    hex.render(svg)
}

/*
const CELL_SIZE = 5; // px
const GRID_COLOR = "#CCCCCC";
const DEAD_COLOR = "#FFFFFF";
const ALIVE_COLOR = "#000000";

const u = Universe.new();

const canvas = document.getElementById('canvas');
canvas.height = (CELL_SIZE + 1) * height + 1;
canvas.width = (CELL_SIZE + 1) * width + 1;
const ctx = canvas.getContext('2d');

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
