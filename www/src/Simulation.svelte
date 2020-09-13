<div class="container">
    <div>
        <div class="canvasDiv">
            <canvas class="mainCanvas" bind:this={canvas} />
            <canvas class="overlayCanvas" bind:this={overlayCanvas} on:mousemove={onMouseMove} />
        </div>
        <div class="button-row">
            <button on:click={onReset}>⏮</button>
            <!-- <button on:clock={onPause}>⏸</button> -->
            <button on:click={onStep}>
                <i class="fas fa-camera"></i> <!-- this icon's 1) style prefix == fas and 2) icon name == camera -->
                1</button>
            <button on:click={onPlayNormal}>▶</button>
            <button on:click={onPlayFast}>▶▶</button>
            <div class="spacer"></div>
            <div class="step">
                {step}
            </div>
        </div>
    </div>
    <div>
        <Sidebar {cell}/>
    </div>
</div>

<style lang="scss">
    .container {
        display: inline-flex;
        flex-direction: row;
        background-color: #CCC;
        border: 2px solid #2E170E;
        border-radius: 5px;
        padding: .5em;
    }
    .canvasDiv {
        position: relative;
    }
    .overlayCanvas {
        position: absolute;
        top: 0;
        left: 0;
        background-color: transparent;
    }
    .mainCanvas {
        background-color: #2E170E;
    }
    .button-row {
        padding-top: 7px;
        display: flex;
    }
    button {
        margin: 0 0.2em 0 0;
    }
    .spacer {
        flex-grow: 1;
    }
    .step {
        color: #2E170E80;
        display: inline-block;
        background-color: transparent;
        border: 1px solid #2E170E40;
        border-radius: 5px;
        min-width: 3.5em;
        padding: .15em .35em;
        align-self: center;
        text-align: center;
   }
</style>

<script lang="ts">
    import Sidebar from './Sidebar.svelte'
    import Simulation from './simulation'
    import { defineGrid, extendHex } from 'honeycomb-grid'
    import { onMount } from 'svelte'
    import { get_size } from "progenitor"
    import type { Hex as HexType } from 'honeycomb-grid'

    let canvas: HTMLCanvasElement
    let overlayCanvas: HTMLCanvasElement
    let cursor = null
    $: cell = cursor ? sim.get_cell_info(cursor.x, cursor.y) : null

    const gridSize = get_size()

    const Hex = extendHex({ size: 8 })
    const Grid = defineGrid(Hex)
    // Grid.parallelogram({ width: 10, height: 10, start: [0, 0], onCreate: renderHex})

    const myGrid = Grid.rectangle({width: gridSize, height: gridSize})

    let ctx: CanvasRenderingContext2D
    let overlayCtx: CanvasRenderingContext2D

    onMount(() => {
        canvas.width = 450
        canvas.height = 388
        console.log('get_size()', get_size())
        ctx = canvas.getContext('2d')

        overlayCanvas.width = canvas.width
        overlayCanvas.height = canvas.height
        overlayCtx = overlayCanvas.getContext('2d')

        onReset()
        onPlayNormal()
    })

    const sim = new Simulation()
    let step = -1
    // const w = new World()
    // w.set_cell(0, 0, 1)

    let intervalId = null
    let playSpeed: 'fast' | 'normal' = 'normal'

    function onReset() {
        sim.reset()
        renderSim()
    }
    function onStep() {
        if (intervalId) {
            clearInterval(intervalId)
            intervalId = null
        }
        sim.tick()
        // requestAnimationFrame(() => renderWorld(w))
        renderSim()
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
        const ticks = playSpeed === 'normal' ? 1 : 8
        for (let i=0; i<ticks; i++) {
            sim.tick()
        }
        // requestAnimationFrame(() => renderWorld(w))
        renderSim()
    }

    function renderSim() {
        // trigger updates (maybe not the most ellegant way...)
        cell = cell
        step = sim.get_step()

        // sim(global)
        const data = sim.update_data()

        ctx.clearRect(0, 0, canvas.width, canvas.height)
        myGrid.forEach(renderHex)

        function renderHex(hex: HexType<object>) {
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

            ctx.save()
            ctx.translate(position.x, position.y)
            ctx.scale(0.97, 0.97)
            ctx.beginPath()
            hex.corners().forEach(({x, y}) => ctx.lineTo(x, y))
            ctx.fillStyle = color
            ctx.fill()
            ctx.restore()
        }
    }


   function onMouseMove({offsetX, offsetY}) {
       const hexCoordinates = Grid.pointToHex(offsetX, offsetY)
       const valid = myGrid.includes(hexCoordinates)
       overlayCtx.clearRect(0, 0, overlayCanvas.width, overlayCanvas.height)
       if (valid) {
           cursor = hexCoordinates
           renderCursorHex(overlayCtx, myGrid.get(hexCoordinates))
       } else {
           cursor = null
       }

       function renderCursorHex(ctx: CanvasRenderingContext2D, hex: HexType<object>) {
           const position = hex.toPoint()
           ctx.save()
           ctx.translate(position.x, position.y)
           ctx.scale(0.97, 0.97)  // FIXME: code duplication
           ctx.beginPath()
           hex.corners().forEach(({x, y}) => ctx.lineTo(x, y))
           ctx.closePath()
           ctx.strokeStyle = '#FFF9'
           ctx.lineWidth = 4
           ctx.stroke()
           ctx.strokeStyle = '#2E170E'
           ctx.lineWidth = 2
           ctx.stroke()
           ctx.restore()
       }
   }
</script>
