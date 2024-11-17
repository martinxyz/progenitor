<script lang="ts">
import { demo_turing, Snapshots } from 'progenitor';
import SimulationPlayer from './SimulationPlayer.svelte'
import Simulation, { rules } from './simulation'
import type { Simulation as ProgenitorSimulation } from 'progenitor'
import type { Rule } from './simulation'
import { renderSim } from './render'
import { Application, GraphicsContext, Graphics, Container, Sprite } from 'pixi.js';
    import { onMount } from 'svelte';

export let rule: Rule

let sim1: ProgenitorSimulation
let sim2: ProgenitorSimulation

sim1 = rule.create()
sim1.steps(300)
sim2 = rule.create()
sim2.steps(3)

let param1 = 0.5;
let canvasContainer: HTMLElement

onMount(async () => {
    const app = new Application()
    await app.init({
        backgroundColor: 0x555555,
        resizeTo: canvasContainer,
        // maybe:
        //  autoDensity: true,
        // resolution: window.devicePixelRatio || 1,
    });
    // const res = app.renderer.height / 100;
    canvasContainer.appendChild(app.canvas);
    let simContainer = renderSim(sim1, app, 600)
    app.stage.addChild(simContainer)
})



</script>

<div>
    <label>
        <span>param1</span>
        <input type="range" bind:value={param1} name="volume" min="0" max="1" step="0.01" />
        <span>
            {param1.toFixed(2)}
        </span>
    </label>
</div>

<div class="canvasContainer" bind:this={canvasContainer}>
</div>

<style lang="scss">
label {
    display: flex;
    gap: .5rem;
    align-items: center;
    input {
        flex-grow: 1;
    }
}
.canvasContainer {
    background-color: #2E170E;
    /* height: 15rem; */
    height: 300px;
    /* width: 15rem; */
}

</style>
