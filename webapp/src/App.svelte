<script lang="ts">
import { onMount } from 'svelte'
import { Application, GraphicsContext, Graphics } from 'pixi.js'

// The issue is that pixijs interactivity doesn't work in prod builds.
//
// Reproduce:
// npm run build && npm run preview
// Hover mouse over yellow circle, it should turn dark but doesn't. (In dev mode
// it does. No event listener gets attached to the canvas in the prod build.)
// (Tested with Firefox. Doesn't reproduce in Chromium.)
//
// When adding some log statements in pixi.js, it becomes clear that
// interactivity doesn't work because the event system/plugin was registered too
// late. More precisely, pixijs/src/events/init.ts was imported too late
// (verified by adding console.log statements at the top).
//
// The init.ts module should have been imported by
// pixijs/src/environment-browser/browserExt.ts (`await import('./browserAll')`),
// But after that await statement, the my console.log in events/init.ts has not
// yet printed (it does print later, after the Application has already started).
//
// Uncommenting one of the imports below fixes interactivity not working in prod builds:
// import 'pixi.js/events'
// import { GraphicsContext, Graphics } from 'pixi.js'
//
// Another thing that also fixes the problem is not importing my WASM module.
// (The module is totally unrelated, no interaction with pixi.js.)
//
// comment out this statement:
import * as progenitor from 'progenitor'

// I use vite-plugin-top-level-await to load the WASM module, according
// to some tutorial. But with modern browsers it's no longer needed. Removing
// it fixes the problem, too! (See comments in vite.config.ts)

let canvasContainer

let app

onMount(async () => {
    app = new Application()

    await app.init({
        backgroundColor: '#223',
    })

    let circle = new Graphics(
        new GraphicsContext().circle(0, 0, 200).fill('yellow'),
    )
    circle.eventMode = 'static'
    circle.on('pointerenter', () => {
        circle.alpha = 0.2
    })

    app.stage.addChild(circle)
    canvasContainer.appendChild(app.canvas)
})
</script>

<div class="canvasContainer" bind:this={canvasContainer}></div>
