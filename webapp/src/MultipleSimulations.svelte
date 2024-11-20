<script lang="ts">
import type { Simulation as ProgenitorSimulation } from 'progenitor'
import type { Rule } from './simulation'
import { renderSim } from './render'
import { Application, Sprite, RenderTexture, Matrix } from 'pixi.js'
import { onMount } from 'svelte'

export let rule: Rule

// let sim1: ProgenitorSimulation
// let sim2: ProgenitorSimulation

let param1 = 0.5
let rows = 4
let cols = 4
let canvasContainer: HTMLElement

const app = new Application()

onMount(async () => {
    await app.init({
        backgroundColor: 0x555555,
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

function onRestart() {
    app.stage.removeChildren()
    for (let i = 0; i < 8 * 8; i++) {
        let sim: ProgenitorSimulation = rule.create()
        sim.steps(300)

        let renderSize = { width: 100, height: 100 }
        let target = RenderTexture.create(renderSize)
        let simContainer = renderSim(sim, app, 100)
        app.renderer.render({
            container: simContainer,
            target,
            transform: Matrix.IDENTITY.clone()
                .translate(-50, -50)
                .scale(2, 2)
                .translate(50, 50),
        })
        let x = (i % 8) * 100
        let y = Math.floor(i / 8) * 100
        app.stage.addChild(new Sprite({ texture: target, x, y }))
    }
}
</script>

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
    <button on:click={onRestart}> Regenerate! </button>
</div>

<label class="row">
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

<div class="canvasContainer" bind:this={canvasContainer}></div>

<style lang="scss">
.row {
    display: flex;
    gap: 0.5rem;
    align-items: center;
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
    height: 800px;
    /* width: 15rem; */
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
