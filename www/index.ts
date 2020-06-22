import { World, get_size } from "progenitor";
import { SVG } from '@svgdotjs/svg.js'
import { defineGrid, extendHex, Hex } from 'honeycomb-grid'

let gridSize = get_size()

const w = new World()
w.make_some_cells()
// w.set_cell(0, 0, 1)
// for (let i = 0; i < 3; i++) w.tick()
// w.set_cell(4, 4, 1)

let data = w.update_data();

// var data = new Uint8Array(gridSize * gridSize);
// for (let i=0; i<gridSize*gridSize; i++) {
//   data[i] = i % 3 == 0 ? 0 : 1;
// }

const svg = SVG().addTo('body').size(450, 450)
const Hex = extendHex({ size: 15 })
const Grid = defineGrid(Hex)
// Grid.parallelogram({ width: 10, height: 10, start: [0, 0], onCreate: renderHex})

function renderHex(hex: Hex<{}>) {
    const position = hex.toPoint()

    let {x, y} = hex.cartesian()
    let idx = y * gridSize + x;
    let d = data[idx];

    let color = '#FFF';
    if (d == 0) color = '#AAA';
    if (d == 1) color = '#171';
    if (d == 2) color = '#BB0';
    if (d == 3) color = '#188';
    if (d == 4) color = '#FAB';

    let corners: number[] = [];
    hex.corners().forEach(({x, y}) => {
      corners.push(x)
      corners.push(y)
    });

    // note: bad performance because each hex triggers layout + style recalculations
    svg
      .polygon(corners)
      .fill(color)
      .stroke({ width: 1, color: '#333' })
      .translate(position.x, position.y)

    /*
    const centerPosition = hex.center().add(position)
    svg
      .text(`${this.x},${this.y}`)
      .font({ size: 12, anchor: 'middle', leading: 1.4, fill: '#69c' })
      .translate(centerPosition.x, centerPosition.y - fontSize)
    */
}

function delay(ms: number) {
  return new Promise<void>(resolve => setTimeout(resolve, ms))
}
(async () => {
  for (let i = 0; i < 300; i++) {
    const k = i % 30;;
    if (k === 0) {
      for (let y = 0; y < gridSize; y++) {
        for (let x = 0; x < gridSize; x++) {
          w.set_cell(y, x, 0);
        }
      }
      w.set_cell(gridSize/2, gridSize/2, 1)
    }
    w.tick(i)
    // w.set_cell(i, 1, 1)
    data = w.update_data()
    svg.clear()
    let grid = Grid.rectangle({
      width: gridSize,
      height: gridSize,
      onCreate: renderHex
    })
    await delay(200);
  }
})()
