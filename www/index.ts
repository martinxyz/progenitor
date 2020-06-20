import { World, get_size } from "progenitor";
// import { memory } from "progenitor/progenitor_bg";

import { SVG, PointArray } from '@svgdotjs/svg.js'
import { defineGrid, extendHex, Hex } from 'honeycomb-grid'

let gridSize = get_size();

const w = new World();
w.make_some_cells();
// console.log('w.count_cells():', w.count_cells());
for (let i = 0; i < 3; i++) w.tick();

const data = w.update_data();
console.log(data);

// const cellsPtr = w.update_data();
// const data = new Uint8Array(memory.buffer, cellsPtr, gridSize * gridSize);

// var data = new Uint8Array(gridSize * gridSize);
// for (let i=0; i<gridSize*gridSize; i++) {
//   data[i] = i % 3 == 0 ? 0 : 1;
// }

let isDone: boolean = false;

const svg = SVG().addTo('body').size(300, 300)

const Hex = extendHex({ size: 15 })

const Grid = defineGrid(Hex)
// Grid.parallelogram({ width: 10, height: 10, start: [0, 0], onCreate: renderHex})
Grid.rectangle({ width: gridSize, height: gridSize, onCreate: renderHex })

  // render(this: Hex, svg: SVGElement) {
function renderHex(hex: Hex<{}>) {
    // hex.render(svg)
    const position = hex.toPoint()
    const centerPosition = hex.center().add(position)

    let {x, y} = hex.cartesian()
    let idx = y * gridSize + x;
    let d = data[idx];

    // console.log(hex.cartesian().x, hex.cartesian().y)
    // console.log(hex.x, hex.y);
    // let color = (hex.x + hex.y) % 3 == 0 ? '#FAB' : '#393';
    let color = (d == 0) ? '#FAB' : '#393';

    let corners: number[] = [];
    hex.corners().forEach(({x, y}) => {
      corners.push(x)
      corners.push(y)
    });

    svg
      // .polygon(hex.corners().map(({ x, y }) => `${x},${y}`))
      // .polygon(hex.corners().map(({ x, y }) => [x, y]))
      .polygon(corners)
      .fill(color)
      .stroke({ width: 1, color: '#333' })
      .translate(position.x, position.y)

    /*
    svg
      .text(`${this.x},${this.y}`)
      .font({ size: 12, anchor: 'middle', leading: 1.4, fill: '#69c' })
      .translate(centerPosition.x, centerPosition.y - fontSize)
    */

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
