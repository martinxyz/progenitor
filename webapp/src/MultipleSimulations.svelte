<script lang="ts">
import type { Simulation as ProgenitorSimulation } from 'progenitor'
import type { Rule } from './simulation'
import { renderSim } from './render'
import { Application, Sprite, RenderTexture, Matrix, Rectangle } from 'pixi.js'
import { onMount } from 'svelte'

export let rule: Rule

// let sim1: ProgenitorSimulation
// let sim2: ProgenitorSimulation

let param1 = 0.5
let rows = 5
let cols = 5
let canvasContainer: HTMLElement

const app = new Application()

onMount(async () => {
    await app.init({
        backgroundColor: '#223',
        resizeTo: canvasContainer,
        // maybe:
        //  autoDensity: true,
        // resolution: window.devicePixelRatio || 1,
    })
    // const res = app.renderer.height / 100;
    canvasContainer.appendChild(app.canvas)

    onRestart()

    // app.stage.addChild(simContainer1);
    // app.stage.addChild(new Sprite(sim1Texture));
})

let renderTextures: RenderTexture[] = []
function onRestart() {
    app.stage.removeChildren()
    renderTextures.forEach((rt) => rt.destroy()) // or reuse...
    let tileSize = Math.floor(
        Math.min(app.screen.width / cols, app.screen.height / rows),
    )
    if (tileSize < 8) tileSize = 8

    for (let row = 0; row < rows; row++) {
        for (let col = 0; col < cols; col++) {
            let sim!: ProgenitorSimulation
            for (let i = 0; i < 15; i++) {
                sim = rule.create()
                sim.steps(400) // this is what takes most time

                let viewport = sim.viewport_hint()
                let cell_types = sim.data(viewport, 0)
                let count = 0
                for (let t of cell_types) {
                    if (t != 0 && t != 255) count += 1
                }
                if (count > 30 && count < 160) break
            }

            const renderSize = tileSize * 2
            let target = RenderTexture.create({
                width: renderSize,
                height: renderSize,
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

            app.stage.addChild(sprite)
        }
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
        step="0.01"
    />
</label>

<div class="row">
    <label>
        <span>rows:</span>
        <input
            type="number"
            bind:value={rows}
            name="rows"
            min="1"
            max="40"
            step="1"
        />
    </label>
    <label>
        <span>cols:</span>
        <input
            type="number"
            bind:value={cols}
            name="cols"
            min="1"
            max="40"
            step="1"
        />
    </label>
    <div class="spacer"></div>
    <button onclick={onRestart}> Generate! </button>
</div>

<div class="canvasContainer" bind:this={canvasContainer}></div>

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
    /* height: 15rem; */
    height: 600px;
    /* width: 15rem; */
    width: 600px;
}
button {
    margin: 0 0.2em 0 0;
    min-width: 3.8em;
    color: #2e170ed5;
}
.spacer {
    width: 0.5rem;
}
</style>
