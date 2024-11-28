<script lang="ts">
import type { Simulation as ProgenitorSimulation } from 'progenitor'
import type { Rule } from './simulation'
import { renderSim } from './render'
import {
    Application,
    Sprite,
    RenderTexture,
    Matrix,
    Rectangle,
    Container,
} from 'pixi.js'
import { onMount } from 'svelte'

let { rule }: { rule: Rule } = $props()

let canvasContainer = $state<HTMLElement | undefined>(undefined)

let param1 = $state(0.5)
let filtering = $state(true)
let columns = $state(5)
let steps = $state(200)
let simConfig = $state('')
let lastError = $state('')
let busy = $state(false)
let large = $state(false)

const app = new Application()

onMount(async () => {
    await app.init({
        backgroundColor: '#223',
        resizeTo: canvasContainer,
        // maybe:
        //  autoDensity: true,
        // resolution: window.devicePixelRatio || 1,
    })
    canvasContainer?.appendChild(app.canvas)
    // app.renderer.on('resize', () => {
    //     console.log('resize!')
    // })

    simConfig = JSON.stringify(rule.default_config, undefined, 2)
    onGenerate()
})

function onGenerate() {
    if (busy) return
    busy = true
    lastError = ''
    app.stage.removeChildren()
    setTimeout(async () => {
        try {
            await restart()
            lastError = ''
        } catch (e: any) {
            console.error(e)
            lastError = '✗ ' + (e?.message || e)
        } finally {
            busy = false
        }
    })
}

function onLarge() {
    large = !large
    if (large) {
        columns *= 2
    } else {
        columns /= 2
    }
    if (columns < 2) columns = 2
    if (columns > 200) columns = 200

    // wait for layout to settle
    setTimeout(() => {
        onGenerate()
    }, 100)
}

let renderTextures: RenderTexture[] = []
async function restart() {
    app.resize() // resize event does not trigger for "large" button
    app.stage.removeChildren()
    renderTextures.forEach((rt) => rt.destroy()) // or reuse...
    renderTextures = []
    let tileSize = Math.floor(app.screen.width / columns)
    if (tileSize < 8) tileSize = 8
    let rows = Math.floor((columns / app.screen.width) * app.screen.height)
    if (rows < 1) rows = 1

    let container = new Container()
    container.pivot.x = (tileSize * columns) / 2
    container.pivot.y = (tileSize * rows) / 2

    container.x = app.screen.width / 2
    container.y = app.screen.height / 2

    for (let row = 0; row < rows; row++) {
        for (let col = 0; col < columns; col++) {
            await new Promise((resolve) => requestAnimationFrame(resolve))

            let sim!: ProgenitorSimulation
            for (let i = 0; i < 15; i++) {
                if (rule.create_with_config) {
                    sim = rule.create_with_config(JSON.parse(simConfig))
                } else {
                    sim = rule.create()
                }

                sim.steps(steps) // this is what takes most time

                if (filtering) {
                    let viewport = sim.viewport_hint()
                    let cell_types = sim.data(viewport, 0)
                    let count = 0
                    for (let t of cell_types) {
                        if (t != 0 && t != 255) count += 1
                    }
                    if (count > 30 && count < 160) break
                } else {
                    break
                }
            }

            const renderSize = tileSize * 2
            let target = RenderTexture.create({
                width: renderSize,
                height: renderSize,
                antialias: true,
                resolution: 2,
                // scaleMode: 'nearest',
            })
            renderTextures.push(target)
            let simContainer = renderSim(sim, app, renderSize)
            app.renderer.render({
                container: simContainer,
                target,
                transform: Matrix.IDENTITY.clone(),
            })
            let x = col * tileSize + tileSize / 2
            let y = row * tileSize + tileSize / 2

            let sprite = new Sprite({ texture: target, x, y })
            sprite.anchor = 0.5
            sprite.eventMode = 'static'
            sprite.cursor = 'pointer'
            sprite.hitArea = new Rectangle(
                -tileSize / 2,
                -tileSize / 2,
                tileSize,
                tileSize,
            )
            sprite.on('pointerenter', () => {
                sprite.alpha = 1.0
                sprite.zIndex = 1
                sprite.scale = 1.3
            })
            sprite.on('pointerleave', () => {
                sprite.alpha = 1.0
                sprite.zIndex = 0
                sprite.scale = 1
            })

            container.addChild(sprite)
        }
        app.stage.addChild(container)
    }
}
</script>

<label class="row" style="display:none;">
    <span>
        param1: {param1.toFixed(2)}
    </span>
    <input
        type="range"
        bind:value={param1}
        name="volume"
        min="0"
        max="1"
        step="0.05"
    />
</label>

{#if rule.default_config}
    <div class="row">
        <textarea bind:value={simConfig}> </textarea>
    </div>
{/if}

<div class="row">
    <label>
        <span>columns:</span>
        <input
            disabled={busy}
            type="number"
            bind:value={columns}
            name="columns"
            min="1"
            max="40"
            step="1"
        />
    </label>
    <label>
        <span>steps:</span>
        <input
            type="number"
            bind:value={steps}
            name="steps"
            min="0"
            max="10000"
            step="1"
        />
    </label>
    <label>
        <input type="checkbox" bind:checked={filtering} />
        <span>filter boring</span>
    </label>
    <div class="spacer"></div>
    <!-- <button onclick={onGenerate} disabled={busy}>Generate!</button> "disabled" steals keyboard focus... -->
    <button onclick={onGenerate} class:busy>Generate!</button>
    <button onclick={onLarge} class:busy>Large</button>
</div>

{#if lastError}
    <div class="row">
        <span class="error" title={lastError}>
            {lastError}
        </span>
    </div>
{/if}

<div class="canvasContainer" class:large bind:this={canvasContainer}></div>

<style lang="scss">
.row {
    display: flex;
    gap: 0.5rem;
    align-items: center;
    padding-bottom: 0.3em;
    input[type='range'] {
        flex-grow: 1;
    }
    input[type='number'] {
        padding-top: 0.1rem;
        padding-bottom: 0.1rem;
    }
}
.canvasContainer {
    background-color: #2e170e;
    height: 800px;
    max-width: 45rem;
    width: 100%;
}
.canvasContainer.large {
    width: 95vw;
    height: 130vh;
    max-width: inherit;
}

button {
    margin: 0 0.2em 0 0;
    min-width: 3.8em;
    &.busy {
        color: gray;
        cursor: wait;
    }
}
.spacer {
    width: 0.5rem;
}
textarea {
    width: 100%;
    height: 10rem;
}
span.error {
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 30rem;
    &.error {
        color: #711;
    }
}
</style>
