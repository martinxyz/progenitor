import { World, get_size } from "progenitor";
import { defineGrid, extendHex, Hex } from 'honeycomb-grid'

let gridSize = get_size()

const w = new World()
// w.set_rules_demo1()
w.set_rules_demo2()
// w.set_cell(0, 0, 1)
// for (let i = 0; i < 3; i++) w.tick()
// w.set_cell(4, 4, 1)

let data = w.update_data()

// var data = new Uint8Array(gridSize * gridSize);
// for (let i=0; i<gridSize*gridSize; i++) {
//   data[i] = i % 3 == 0 ? 0 : 1;
// }

const canvas = document.getElementById('hex-canvas') as HTMLCanvasElement
canvas.width = 500
canvas.height = 500
const ctx = canvas.getContext('2d')!


const Hex = extendHex({ size: 8 })
const Grid = defineGrid(Hex)
// Grid.parallelogram({ width: 10, height: 10, start: [0, 0], onCreate: renderHex})

function renderHex(hex: Hex<{}>) {
    const position = hex.toPoint()

    let {x, y} = hex.cartesian()
    let idx = y * gridSize + x
    let d = data[idx]

    let color = '#FFF'
    if (d == 0) color = '#AAA'
    if (d == 1) color = '#171'
    if (d == 2) color = '#BB0'
    if (d == 3) color = '#188'
    if (d == 4) color = '#FAB'

    let corners: number[] = []
    ctx.save()
    ctx.translate(position.x, position.y)
    ctx.scale(0.98, 0.98)
    ctx.beginPath()
    hex.corners().forEach(({x, y}) => ctx.lineTo(x, y))
    ctx.fillStyle = color
    ctx.fill()
    ctx.restore()



    // note: bad performance because each hex triggers layout + style recalculations
    // svg
    //   .polygon(corners)
    //   .fill(color)
    //   .stroke({ width: 1, color: '#333' })
    //   .translate(position.x, position.y)
}

function delay(ms: number) {
  return new Promise<void>(resolve => setTimeout(resolve, ms))
}
(async () => {
  for (let i = 0; i < 3000; i++) {
    const k = i % 500
    if (k === 0) {
      for (let y = 0; y < gridSize; y++) {
        for (let x = 0; x < gridSize; x++) {
          w.set_cell(y, x, 0)
        }
      }
      w.set_cell(gridSize/2, gridSize/2, 1)
      w.set_cell(gridSize/2+3, gridSize/2-0, 1)
      w.set_cell(gridSize/2+1, gridSize/2-8, 1)
      w.set_cell(gridSize/2+3, gridSize/2-2, 1)
    } else {
      w.tick()
    }
    // w.set_cell(i, 1, 1)
    data = w.update_data()
    ctx.fillStyle = '#000F'
    ctx.fillRect(0, 0, canvas.width, canvas.height)
    let grid = Grid.rectangle({
      width: gridSize,
      height: gridSize,
      onCreate: renderHex
    })
    await delay(200)
  }
})()
