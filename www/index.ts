import { World, get_size } from "progenitor"
import { defineGrid, extendHex, Hex } from 'honeycomb-grid'

let gridSize = get_size()

// var data = new Uint8Array(gridSize * gridSize)
// for (let i=0; i<gridSize*gridSize; i++) {
//   data[i] = i % 3 == 0 ? 0 : 1
// }

const canvas = document.getElementById('hex-canvas') as HTMLCanvasElement
canvas.width = 500
canvas.height = 500
const ctx = canvas.getContext('2d')!

const Hex = extendHex({ size: 8 })
const Grid = defineGrid(Hex)
// Grid.parallelogram({ width: 10, height: 10, start: [0, 0], onCreate: renderHex})

// function delay(ms: number) {
//   return new Promise<void>(resolve => setTimeout(resolve, ms))
// }
function main() {
  const w = new World()
  // w.set_rules_demo1()
  // w.set_rules_demo2()
  w.set_rules_demo3()
  // w.set_cell(0, 0, 1)
  // for (let i = 0; i < 3; i++) w.tick()
  // w.set_cell(4, 4, 1)

  document.getElementById('btn-reset')?.addEventListener('click', onReset)
  document.getElementById('btn-step')?.addEventListener('click', onStep)
  document.getElementById('btn-play')?.addEventListener('click', onPlayNormal)
  document.getElementById('btn-play-fast')?.addEventListener('click', onPlayFast)

  let intervalId: NodeJS.Timeout | null = null
  let playSpeed: 'fast' | 'normal' = 'normal'
  onReset()
  onPlayNormal()

  function onReset() {
    resetSimulation(w)
    renderWorld(w)
  }
  function onStep() {
    if (intervalId) {
      clearInterval(intervalId)
      intervalId = null
    }
    w.tick()
    // requestAnimationFrame(() => renderWorld(w))
    renderWorld(w)
  }
  function onPlayNormal() {
    playSpeed = 'normal'
    play()
  }
  function onPlayFast() {
    playSpeed = 'fast'
    play()
  }
  function stop() {
    if (intervalId) {
      clearInterval(intervalId)
      intervalId = null
    }
  }
  function play() {
    stop()
    intervalId = setInterval(intervalCallback, playSpeed === 'normal' ? 300 : 100)
  }

  function intervalCallback() {
    console.log('intervalCallback')
    const ticks = playSpeed === 'normal' ? 1 : 8
    for (let i=0; i<ticks; i++) {
      w.tick()
    }
    // requestAnimationFrame(() => renderWorld(w))
    renderWorld(w)
  }
}
main()

function resetSimulation(w: World) {
    for (let y = 0; y < gridSize; y++) {
        for (let x = 0; x < gridSize; x++) {
            w.set_cell(y, x, 0)
        }
    }
    w.set_cell(gridSize / 2, gridSize / 2, 1)
    w.set_cell(gridSize / 2 + 3, gridSize / 2 - 0, 1)
    w.set_cell(gridSize / 2 + 1, gridSize / 2 - 8, 1)
    w.set_cell(gridSize / 2 + 3, gridSize / 2 - 2, 1)
}

function renderWorld(w: World) {
  const data = w.update_data()
  ctx.clearRect(0, 0, canvas.width, canvas.height)
  Grid.rectangle({
    width: gridSize,
    height: gridSize,
    onCreate: renderHex
  })

  function renderHex(hex: Hex<{}>) {
    const position = hex.toPoint()

    let {x, y} = hex.cartesian()
    let idx = y * gridSize + x
    let d = data[idx]

    // let color = '#FFF'
    let color = '#188'
    if (d == 0) color = '#AAA'
    if (d == 1) color = '#292'
    if (d == 2) color = '#357'
    if (d == 3) color = '#188'
    if (d == 4) color = '#FFF'
    if (d == 5) color = '#799'

    let corners: number[] = []
    ctx.save()
    ctx.translate(position.x, position.y)
    ctx.scale(0.97, 0.97)
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
}
