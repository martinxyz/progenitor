<script lang="ts">
    import Sidebar from './Sidebar.svelte'
    import type Simulation from './simulation'
    import type { Viewport } from 'progenitor';

    import { onMount } from 'svelte'
    import { createHexDimensions, defineHex, Grid, rectangle, type HexCoordinates } from 'honeycomb-grid'
    import type { Hex as HexType } from 'honeycomb-grid'
    import Color from 'colorjs.io'

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
    $: cellText = (sim && cursor) ? sim.get_cell_text(
        // FIXME: this leads to correct behaviour, but should be solved elsewhere
        cursor.col + viewportCol + (viewportRow % 2 == 0 ? 0 : (cursor.row % 2)), cursor.row + viewportRow
    ) : null

    let showEnergy = false
    let showDirection = true

    let Hex: typeof HexType
    let myGrid: Grid<HexType>
    let viewportCol = 0
    let viewportRow = 0

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

        // zoom to keep hex size large enough to keep things pleasant to look/point at
        const maxHexes = 48
        if (viewport.width > maxHexes) {
            viewport.col += Math.floor((viewport.width - maxHexes) / 2)
            viewport.width = maxHexes
        }
        if (viewport.height > maxHexes) {
            viewport.row += Math.floor((viewport.height - maxHexes) / 2)
            viewport.height = maxHexes
        }
        viewportCol = viewport.col
        viewportRow = viewport.row

        updateHexgrid(viewport, canvas)
        const detailed = (Hex.settings.dimensions.xRadius > 10)

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
            if (ct == 0) color = '#efe'
            if (ct == 1) color = '#b57'
            if (ct == 2) color = '#997'
            if (ct == 3) color = '#380'
            if (ct == 4) color = '#651'
            if (ct == 5) color = '#87c'

            if (showEnergy && e !== 255) {
                let c = new Color(color)
                c.lab.l += (e - 2) * 2
                color = c.toString({format: "hex"})
            }

            ctx.save()
            if (detailed) {
                // gap between hexes
                ctx.translate(hex.x, hex.y)
                ctx.scale(0.97, 0.97)
                ctx.translate(-hex.x, -hex.y)
            }
            ctx.beginPath()
            hex.corners.forEach(({x, y}) => ctx.lineTo(x, y))

            ctx.fillStyle = color
            ctx.fill()
            ctx.restore()

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
            // start: {col: viewport.col, row: viewport.row},
            width: viewport.width, height: viewport.height
        }))
    }

    function cursorToHex({clientX, clientY}): HexType {
        let rect = canvas.getBoundingClientRect();
        const x = (clientX - rect.left) / (rect.right - rect.left) * canvas.width;
        const y = (clientY - rect.top) / (rect.bottom - rect.top) * canvas.height;
        const hexCoordinates = myGrid.pointToHex({x, y})
        if (myGrid.hasHex(hexCoordinates)) {
            return hexCoordinates
        } else {
            return null
        }
    }

    function onMouseMove(event: MouseEvent) {
        cursorHover = cursorToHex(event)
    }

    function onMouseLeave() {
        cursorHover = null
    }

    function onClick(event: MouseEvent) {
        const p = cursorToHex(event)
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
