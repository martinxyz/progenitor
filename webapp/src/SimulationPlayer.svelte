<script lang="ts">
    import Sidebar from './Sidebar.svelte'
    import type Simulation from './simulation'
    import { defineGrid, extendHex } from 'honeycomb-grid'
    import { onMount } from 'svelte'
    import { get_size } from 'progenitor'
    import type { Hex as HexType, PointCoordinates } from 'honeycomb-grid'

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
    let showDirection = false

    const gridSize = get_size()

    const Hex = extendHex({ size: 8 })
    const Grid = defineGrid(Hex)
    // Grid.parallelogram({ width: 10, height: 10, start: [0, 0], onCreate: renderHex})

    const myGrid = Grid.rectangle({width: gridSize, height: gridSize})

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
        console.log('get_size()', get_size())
        ctx = canvas.getContext('2d')

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
        if (!sim) return
        // console.log('renderSim')

        // to trigger updates (maybe not the most ellegant way...)
        step = sim.get_step_no()

        const [data_cell_type, data_energy, data_direction] = sim.get_data()
        myGrid.forEach(renderHex)

        function renderHex(hex: HexType<object>) {
            const position = hex.toPoint()

            let {x, y} = hex.cartesian()
            let idx = y * gridSize + x
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
            ctx.translate(position.x, position.y)
            ctx.scale(0.97, 0.97)
            ctx.beginPath()
            hex.corners().forEach(({x, y}) => ctx.lineTo(x, y))
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
                ctx.translate(position.x, position.y)
                ctx.scale(0.97, 0.97)
                ctx.beginPath()
                ctx.arc(hex.center().x, hex.center().y, 4.0, 0, 2*Math.PI)
                ctx.fillStyle = color
                ctx.fill()
                ctx.restore()
            }
            if (showDirection && dir !== 255) {
                ctx.save()
                ctx.translate(position.x, position.y)
                ctx.translate(hex.center().x, hex.center().y)
                ctx.rotate((dir+4) / 6 * 2*Math.PI)
                ctx.translate(4.0, 0)
                ctx.beginPath()
                ctx.arc(0, 0, 1.5, 0, 2*Math.PI)
                ctx.fillStyle = '#000'
                ctx.fill()
                ctx.restore()
            }
        }
    }

    function offsetToHex(offsetX: number, offsetY: number)  {
        const hexCoordinates = Grid.pointToHex(offsetX, offsetY)
        if (myGrid.includes(hexCoordinates)) {
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
        if (p && p.equals(cursorSelected)) {
            cursorSelected = null
            cursorHover = null
        } else {
            cursorSelected = p
        }
    }

    function renderCursors(selected: PointCoordinates, hover: PointCoordinates) {
        if (!overlayCtx) return
        overlayCtx.clearRect(0, 0, overlayCanvas.width, overlayCanvas.height)
        if (selected) {
            renderCursorHex(overlayCtx, myGrid.get(selected))
        } else if (hover) {
            renderCursorHex(overlayCtx, myGrid.get(hover))
        }
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
