<script lang="ts">
import type { Simulation as ProgenitorSimulation } from 'progenitor'
import type { Rule } from './simulation'
import { renderSim } from './render'
import { Application, Sprite, RenderTexture, Matrix, Rectangle, Container } from 'pixi.js'
import { onMount, onDestroy } from 'svelte'

let { rule }: { rule: Rule } = $props()

let canvasContainer = $state<HTMLElement | undefined>(undefined)
let fullscreenDiv: HTMLElement

let param1 = $state(0.5)
let filtering = $state(true)
let columns = $state(6)
let steps = $state(200)
let simConfig = $state('')
let lastError = $state('')
let busy = $state(false)
let isFullscreen = $state(false)

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

let renderTextures: RenderTexture[] = []
function destroyScene() {
    renderTextures.forEach((rt) => rt.destroy(true)) // or reuse them...?
    renderTextures = []

    for (let child of app.stage.children) {
        child.destroy(true)
    }
    app.stage.removeChildren()

    // FIXME: with columns=33 we are still leaking some ~100 MB on JS heap per restart (most
    // inside some ArrayBuffer)
}

onDestroy(() => {
    destroyScene()
    app.destroy(true)
})

function onGenerate() {
    if (busy) return
    busy = true
    lastError = ''
    app.stage.removeChildren() // or destroy?
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

async function onFullscreenButton() {
    if (document.fullscreenElement !== fullscreenDiv) {
        await fullscreenDiv.requestFullscreen()
        onFullscreenChange()
        isFullscreen = true
    } else {
        await document.exitFullscreen()
        onFullscreenChange()
        isFullscreen = false
    }
}
// event not available on all browsers
async function onFullscreenChange() {
    let isFullscreenOld = isFullscreen
    isFullscreen = document.fullscreenElement === fullscreenDiv
    if (isFullscreenOld === isFullscreen) return

    // wait for layout to settle, then re-generate
    await new Promise((resolve) => setTimeout(resolve, 400))
    onGenerate()
}

async function restart() {
    destroyScene()
    app.resize() // resize event does not trigger for "fullscreen" button
    let tileSize = Math.floor(app.screen.width / columns)
    if (tileSize < 8) tileSize = 8
    let rows = Math.floor((columns / app.screen.width) * app.screen.height)
    if (rows < 1) rows = 1

    let container = new Container()
    container.pivot.x = (tileSize * columns) / 2
    container.pivot.y = (tileSize * rows) / 2

    container.x = app.screen.width / 2
    container.y = app.screen.height / 2

    let stepsSinceLastFrame = 0

    for (let row = 0; row < rows; row++) {
        for (let col = 0; col < columns; col++) {
            let sim!: ProgenitorSimulation
            for (let i = 0; i < 15; i++) {
                if (rule.create_with_config) {
                    sim = rule.create_with_config(JSON.parse(simConfig))
                } else {
                    sim = rule.create()
                }

                sim.steps(steps) // this is what takes most time

                stepsSinceLastFrame += steps
                if (stepsSinceLastFrame > 1000) {
                    await new Promise(requestAnimationFrame)
                    stepsSinceLastFrame = 0
                }

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
            simContainer.destroy({
                children: true,
                texture: true,
                textureSource: true,
            }) // or reuse?
            let x = col * tileSize + tileSize / 2
            let y = row * tileSize + tileSize / 2

            let sprite = new Sprite({ texture: target, x, y })
            sprite.anchor = 0.5
            sprite.eventMode = 'static'
            // sprite.cursor = 'pointer'
            sprite.hitArea = new Rectangle(-tileSize / 2, -tileSize / 2, tileSize, tileSize)
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

<svelte:document onfullscreenchange={onFullscreenChange} />

<label class="row" style="display:none;">
    <span>
        param1: {param1.toFixed(2)}
    </span>
    <input type="range" bind:value={param1} name="volume" min="0" max="1" step="0.05" />
</label>

{#if rule.default_config}
    <div class="row">
        <textarea spellcheck="false" bind:value={simConfig}> </textarea>
    </div>
{/if}

<div bind:this={fullscreenDiv} class="fullscreenDiv" class:isFullscreen>
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
            <input type="number" bind:value={steps} name="steps" min="0" max="10000" step="1" />
        </label>
        <label>
            <input type="checkbox" bind:checked={filtering} />
            <span>filter boring</span>
        </label>
        <div class="spacer"></div>
        <button onclick={onGenerate} class:busy>Generate!</button>
        <div class="spacer-grow"></div>
        <button
            title="fullscreen"
            aria-label="fullscreen"
            onclick={onFullscreenButton}
            class="fas fa-up-right-and-down-left-from-center"
        ></button>
    </div>

    {#if lastError}
        <div class="row">
            <span class="error" title={lastError}>
                {lastError}
            </span>
        </div>
    {/if}

    <div class="canvasContainer" bind:this={canvasContainer}></div>
</div>

<style lang="scss">
.row {
    display: flex;
    gap: 0.5rem;
    /* padding-bottom: 0.3em; */
    margin-bottom: 0.3em;
    input[type='range'] {
        flex-grow: 1;
    }
    input[type='number'] {
        padding-top: 0.1rem;
        padding-bottom: 0.1rem;
    }
    align-content: space-between;
}

.fullscreenDiv {
    display: flex;
    flex-direction: column;
    background-color: #ccc;
}
.fullscreenDiv.isFullscreen .row {
    margin: 0.3em;
}

.canvasContainer {
    height: 38rem;
    flex-grow: 1;
    background-color: #2e170e;
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
.spacer-grow {
    flex-grow: 1;
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
