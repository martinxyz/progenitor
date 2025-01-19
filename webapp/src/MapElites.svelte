<script lang="ts">
import type { Rule } from './simulation'
import * as progenitor from 'progenitor'
import { onMount } from 'svelte'

interface Solution {
    measures: Float32Array,
    seed: bigint,
}

let map_bins: (Solution | null)[][] = $state([])
let bins_rows: number[] = $state([])
let bins_cols: number[] = $state([])

let {
    selectHandler = () => {},
}: {
    selectHandler: (rule: Rule) => void
} = $props()

$effect(() => {
})

onMount(() => {
    initMap()
    setTimeout(evolve, 300)
})

let evolve_step = 0n
function evolve() {
    evolve_step += 1n
    let measures = progenitor.measure_hive(evolve_step)
    console.log('measure_hive', evolve_step, progenitor.measure_hive(evolve_step))
    let row = bins_rows.findLastIndex(value => measures[0] > value)
    let col = bins_rows.findLastIndex(value => measures[1] > value)
    console.log('row, col:', row, col)
    if (row >= 0 && col >= 0) {
        if (!map_bins[row][col]) {
            map_bins[row][col] = { seed: evolve_step, measures }
        }
    }
    setTimeout(evolve, 300)
}

async function initMap() {
    bins_cols = Array.from({ length: 20 }, (_, i) => i * 5.0)
    bins_rows = Array.from({ length: 20 }, (_, i) => i * 5.0)
    // bins_rows.reverse()

    map_bins = []
    for (let _ of bins_rows) {
        let r: any[] = []
        map_bins.push(r)
        for (let _ of bins_cols) {
            r.push(null)
        }
    }
}

function loadbin(bin: bigint | null) {
    if (!bin) return
    let rule2: Rule = {
        label: '(selected from map)',
        create: () => {
            return progenitor.demo_hive(bin)
        },
    }
    selectHandler(rule2)
}
</script>

<div class="table">
    {#each bins_rows as row, row_idx}
        <div class="row">
            {#each bins_cols as col, col_idx}
                {@const bin = map_bins[row_idx][col_idx]}
                <div
                    class="cell"
                    class:full={bin != null}
                    onclick={() => { if (bin) loadbin(bin.seed) }}
                >
                    <div title={`${row}, ${col}, ${bin?.measures}`} class="inner"></div>
                </div>
            {/each}
        </div>
    {/each}
</div>

<style lang="scss">
.table {
    width: 100%;
    height: 20em;
    display: flex;
    flex-direction: column;
}
.row {
    flex-grow: 1;
    display: flex;
    flex-direction: row;
}
.cell {
    flex-grow: 1;
    padding: 1px;
}
.inner {
    background-color: #2e170e80;
    width: 100%;
    height: 100%;
}
.full > .inner {
    background-color: #000;
}
.full:hover {
    cursor: pointer;
    & > .inner {
        background-color: #055;
    }
}
</style>
