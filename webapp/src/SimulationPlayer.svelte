<script lang="ts">
    import Sidebar from './Sidebar.svelte'
    import type Simulation from './simulation'
    import { createHexDimensions, defineHex, Grid, rectangle, type HexCoordinates } from 'honeycomb-grid'
    import type { Viewport } from 'progenitor';

    import { onMount } from 'svelte'
    import type { Hex as HexType } from 'honeycomb-grid'

    export let sim: Simulation

    let autoplay = true
    $: {
        if (sim && autoplay) {
            autoplay = false
            onPlay()
        }
    }

    let canvas: HTMLCanvasElement
    let overlayCanvas: HTMLCanvasElement
    let cursorHover = null
    let cursorSelected = null
    $: renderCursors(cursorSelected, cursorHover)
    $: cursor = cursorSelected || cursorHover
    $: cellText = (sim && cursor) ? sim.get_cell_text(cursor.x, cursor.y) : null

    let showEnergy = false
    let showDirection = true

    let Hex: typeof HexType
    let myGrid: Grid<HexType>

    let ctx: CanvasRenderingContext2D
    let overlayCtx: CanvasRenderingContext2D

    $: {
        // rendering triggers
        sim
        showEnergy
        showDirection
        // This causes an infinite re-triggering of the rendering:
        //   requestAnimationFrame(renderSim)
        renderSim()
    }

    onMount(() => {
        canvas.width = 450
        canvas.height = 388
        ctx = canvas.getContext('2d', {alpha: false})

        overlayCanvas.width = canvas.width
        overlayCanvas.height = canvas.height
        overlayCtx = overlayCanvas.getContext('2d')
    })

    let step = -1
    let intervalId = null
    let playSpeed = 1

    $: playing = (intervalId != null)

    function onRestart() {
        sim.restart()
        sim = sim
        // if already playing, restart the timer
        if (!playing) play()
    }
    function onPlay() {
        play()
    }
    function onPause() {
        stop()
    }
    function onStep() {
        stop()
        sim.steps(1)
        sim = sim
    }
    function onUndoStep() {
        stop()
        sim.step_undo()
        sim = sim
    }
    function onPlaySpeed() {
        if (playSpeed == 1) {
            playSpeed = 3
        } else if (playSpeed < 10_000) {
            playSpeed *= 10
        } else {
            playSpeed = 1
        }
        if (playing) play()
    }
    function stop() {
        if (intervalId) {
            clearInterval(intervalId)
            intervalId = null
        }
    }
    function play() {
        stop()
        intervalId = setInterval(intervalCallback, playSpeed == 1 ? 300 : 100)
    }

    function intervalCallback() {
        try {
            if (document.hidden) return
            sim.steps(playSpeed == 1 ? playSpeed : playSpeed / 3)
            sim = sim
        } catch (e) {
            stop()
            throw e
        }
    }

    function onKey(ev: KeyboardEvent) {
        const fn = {
            'ArrowRight': onStep,
            'ArrowLeft': onUndoStep,
            'h': onStep,
            'l': onUndoStep,
            'Backspace': onRestart,
        }[ev.key]
        if (fn) {
            fn()
            ev.preventDefault()
            ev.stopPropagation()
        }
    }

    function renderSim() {
        if (!ctx) return
        ctx.clearRect(0, 0, canvas.width, canvas.height)
        // debug what gets rendered:
        // ctx.fillStyle = '#233'
        // ctx.fillRect(0, 0, canvas.width, canvas.height)

        if (!sim) return

        // to trigger updates (maybe not the most ellegant way...)
        step = sim.get_step_no()

        let viewport = sim.get_data_viewport()

        updateHexgrid(viewport, canvas)

        // TODO: names, not indices
        // let data_cell_type = sim.get_data(viewport, 'cell_type')
        // let data_energy = sim.get_data(viewport, 'energy')
        // let data_direction = sim.get_data(viewport, 'direction')
        let data_cell_type = sim.get_data(viewport, 0)
        let data_energy = sim.get_data(viewport, 1)
        let data_direction = sim.get_data(viewport, 2)

        for (const hex of myGrid) {
            let idx = hex.row * viewport.width + hex.col
            let ct = data_cell_type[idx]
            let e = data_energy[idx]
            let dir = data_direction[idx]

            let color = '#000'
            if (ct == 0) color = '#AAA'
            if (ct == 1) color = '#292'
            if (ct == 2) color = '#268'
            if (ct == 3) color = '#188'
            if (ct == 4) color = '#843'
            if (ct == 5) color = '#87c'

            ctx.save()
            ctx.translate(hex.x, hex.y)
            ctx.scale(0.97, 0.97)
            ctx.translate(-hex.x, -hex.y)
            ctx.beginPath()
            hex.corners.forEach(({x, y}) => ctx.lineTo(x, y))

            ctx.fillStyle = color
            ctx.fill()
            if (showEnergy && e !== 255) {
                ctx.fillStyle = '#555A'
                ctx.fill()
            }
            ctx.restore()

            if (showEnergy && e !== 255) {
                if (e === 0) color = '#000'
                if (e === 1) color = '#880'
                if (e === 2) color = '#AA0'
                if (e === 3) color = '#CC0'
                if (e === 4) color = '#DD0'
                if (e === 5) color = '#EE2'
                if (e >= 6) color = '#FF6'
                ctx.save()
                ctx.translate(hex.x, hex.y)
                ctx.scale(hex.dimensions.xRadius, hex.dimensions.yRadius)
                ctx.beginPath()
                ctx.arc(0, 0, 0.45, 0, 2*Math.PI)
                ctx.fillStyle = color
                ctx.fill()
                ctx.restore()
            }
            if (showDirection && dir !== 255) {
                ctx.save()
                ctx.translate(hex.x, hex.y)
                ctx.scale(hex.dimensions.xRadius, hex.dimensions.yRadius)
                ctx.rotate((dir+4) / 6 * 2*Math.PI)
                ctx.translate(0.55, 0)
                ctx.beginPath()
                ctx.arc(0, 0, 0.2, 0, 2*Math.PI)
                ctx.fillStyle = '#000'
                ctx.fill()
                ctx.restore()
            }
        }

        // FIXME: change API such that this is not needed
        viewport.free()
    }

    let lastHexgridInputs = [0, 0, 0, 0]
    function updateHexgrid(viewport: Viewport, canvas: HTMLCanvasElement) {
        // ugh, change detection
        let hexgridInputs = [viewport.width, viewport.height, canvas.width, canvas.height]
        if (lastHexgridInputs.every((value, idx) => value === hexgridInputs[idx])) {
            return
        }
        lastHexgridInputs = hexgridInputs

        // Calculate hex size to fit all hexes in the viewport fully on the canvas.
        let measuringGrid = new Grid(
            defineHex({ dimensions: 1 }),
            rectangle({width: viewport.width, height: viewport.height})
        )
        let sizeX = canvas.width / measuringGrid.pixelWidth
        let sizeY = canvas.height / measuringGrid.pixelHeight
        let size = Math.min(sizeX, sizeY)
        let paddingX = (sizeX - size) * measuringGrid.pixelWidth / 2
        let paddingY = (sizeY - size) * measuringGrid.pixelHeight / 2
        // Maybe also render adjacent hexes...? (Usefulness depends on simulation.)
        // viewport.height += 2
        // viewport.width += 2
        // viewport.col -= 1
        // viewport.row -= 1
        // On the other hand, if the viewport can zoom and and pan, we might
        // want to animate that via CSS (because that is accelerated)... later.

        Hex = defineHex({
            dimensions: createHexDimensions(size),
            // translate the rendered grid in pixel-space
            // origin: 'topLeft',  // same result as below, but no padding
            origin: {
                x: -size*Math.sqrt(3)/2 - paddingX,
                y: -size - paddingY
            }
        })
        myGrid = new Grid(Hex, rectangle({
            width: viewport.width, height: viewport.height
        }))
    }

    function offsetToHex(offsetX: number, offsetY: number): HexType {
        const hexCoordinates = myGrid.pointToHex({x: offsetX, y: offsetY})
        if (myGrid.hasHex(hexCoordinates)) {
            return hexCoordinates
        } else {
            return null
        }
    }

    function onMouseMove({offsetX, offsetY}) {
        cursorHover = offsetToHex(offsetX, offsetY)
    }

    function onMouseLeave() {
        cursorHover = null
    }

    function onClick({offsetX, offsetY}) {
        const p = offsetToHex(offsetX, offsetY)
        if (cursorSelected != null && p && p.equals(cursorSelected)) {
            cursorSelected = null
            cursorHover = null
        } else {
            cursorSelected = p
        }
    }

    function renderCursors(selected: HexCoordinates, hover: HexCoordinates) {
        if (!overlayCtx) return
        overlayCtx.clearRect(0, 0, overlayCanvas.width, overlayCanvas.height)
        if (selected) {
            renderCursorHex(overlayCtx, myGrid.getHex(selected))
        } else if (hover) {
            renderCursorHex(overlayCtx, myGrid.getHex(hover))
        }
    }

    function renderCursorHex(ctx: CanvasRenderingContext2D, hex: HexType) {
        ctx.save()
        ctx.beginPath()
        hex.corners.forEach(({x, y}) => ctx.lineTo(x, y))
        ctx.closePath()
        ctx.strokeStyle = '#FFF9'
        ctx.lineWidth = 4
        ctx.stroke()
        ctx.strokeStyle = '#2E170E'
        ctx.lineWidth = 2
        ctx.stroke()
        ctx.restore()
    }
</script>

<div class="host" on:keydown={onKey}>
    <div>
        <div class="canvasDiv">
            <canvas class="mainCanvas" bind:this={canvas} />
            <canvas
                class="overlayCanvas"
                bind:this={overlayCanvas}
                on:mousemove={onMouseMove}
                on:mouseleave={onMouseLeave}
                on:click={onClick}
            />
        </div>
        <div class="button-row">
            <button on:click={onRestart} title="Restart (Backspace)">
                <i class="fas fa-sync"></i>
            </button>
            <button on:click={onUndoStep} title="Step Back (Arrow Left)">
                <i class="fas fa-step-backward"></i>
            </button>
            <button on:click={onStep} title="Single Step (Arrow Right)">
                <i class="fas fa-step-forward"></i>
            </button>
            {#if playing}
                <button on:click={onPause} title="Pause">
                    <i class="fas fa-pause"></i>
                </button>
            {:else}
                <button on:click={onPlay} title="Play">
                    <i class="fas fa-play"></i>
                </button>
            {/if}
            <button on:click={onPlaySpeed} title="Play Speed">
                {#if playSpeed < 2000}
                    x{playSpeed}
                {:else}
                    x{Math.round(playSpeed / 1000)}k
                {/if}
            </button>
            <div class="spacer"></div>
            <div class="step">
                {step}
            </div>
        </div>
    </div>
    <div>
        <Sidebar {cellText} bind:showEnergy bind:showDirection/>
    </div>
</div>

<style lang="scss">
    .host {
        display: inline-flex;
        flex-direction: row;
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
        min-width: 3.8em;
        color: #2E170ED5;
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
