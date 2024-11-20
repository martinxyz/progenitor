import * as progenitor from 'progenitor'
import type { Simulation as ProgenitorSimulation } from 'progenitor'
import type { Viewport } from 'progenitor'
import Color from 'colorjs.io'
import {
    Application,
    GraphicsContext,
    Graphics,
    Container,
    Sprite,
} from 'pixi.js'
const TAU = 2 * Math.PI

export function renderSim(
    sim: ProgenitorSimulation,
    app: Application,
    size: number,
) {
    let grid = new Container()

    let viewport = sim.viewport_hint()
    // zoom to keep hex size large enough to keep things pleasant to look/point at
    // const maxHexes = 48
    // if (viewport.width > maxHexes) {
    //     viewport.col += Math.floor((viewport.width - maxHexes) / 2)
    //     viewport.width = maxHexes
    // }
    // if (viewport.height > maxHexes) {
    //     viewport.row += Math.floor((viewport.height - maxHexes) / 2)
    //     viewport.height = maxHexes
    // }
    let get_data = (channel: number): Uint8Array => {
        // contract: use it immediately, may be invalid the next time any wasm is called
        return sim.data(viewport, channel)
    }
    // let hexSize = app.canvas.width / (viewport.width + 0.5) / Math.sqrt(3)
    let hexSize = size / (viewport.width + 0.5) / Math.sqrt(3)
    let hexWidth = hexSize * Math.sqrt(3)
    grid.x = hexSize
    grid.y = hexSize

    const detailed = hexWidth > 17
    let hexagonContext = new GraphicsContext()
        .regularPoly(0, 0, detailed ? hexSize * 0.9 : hexSize, 6)
        .fill('white')

    let data_cell_type = get_data(0)
    let data_energy = get_data(1)
    let data_direction = get_data(2)

    // iterate in odd-r offset coordinates
    for (let row = 0; row < viewport.height; row++) {
        for (let col = 0; col < viewport.width; col++) {
            let idx = row * viewport.width + col
            let ct = data_cell_type[idx]
            let e = data_energy[idx]
            let dir = data_direction[idx]

            // if (ct == 255) continue;  // map border
            if (ct == 255 || ct == 0) continue // map border or air

            let color = '#000'
            if (ct == 0) color = '#efe'
            if (ct == 1) color = '#b57'
            if (ct == 2) color = '#997'
            if (ct == 3) color = '#380'
            if (ct == 4) color = '#651'
            if (ct == 5) color = '#87c'

            let showEnergy = true
            if (showEnergy && e !== 255) {
                let c = new Color(color)
                c.lab.l += (e - 12) * 1.2
                color = c.toString({ format: 'hex' })
            }

            let hex = new Graphics(hexagonContext)
            grid.addChild(hex)

            // odd-r to axial
            let q = col - (row - (row & 1)) / 2
            let r = row
            hex.x = q * hexWidth + (hexWidth / 2) * r
            hex.y = (r * hexSize * 3) / 2

            hex.tint = color

            // hex.eventMode = 'static'
            // hex.on('pointerenter', () => {
            //     hex.alpha = 0.3
            // })
            // hex.on('pointerleave', () => {
            //     hex.alpha = 1.0
            // })
            // hex.on('pointerdown', () => {
            //     hex.alpha = 0.3
            // })
        }
    }

    let t = 0
    app.ticker.add(() => {
        t += 0.05
        // grid.scale = 1.0 + 0.1 * Math.cos(t*.03)
        // for (let child of grid.children) {
        //     child.tint = { r: 80, g: 80, b: Math.round(Math.random() * 65) }
        // }
    })

    // FIXME: change API such that this is not needed
    viewport.free()

    return grid
}
