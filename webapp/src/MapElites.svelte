<script lang="ts">
import type { Rule } from './simulation'
import * as progenitor from 'progenitor'
import { onMount } from 'svelte'
import { archive_bin, archive_cols, archive_rows, type Solution } from './archive'

let map_bins: (Solution | null)[] = $state(Array(archive_rows * archive_cols).fill(null))

let {
    selectHandler = () => {},
}: {
    selectHandler: (rule: Rule) => void
} = $props()

$effect(() => {
})

onMount(() => {
    setTimeout(evolve, 300)
})

let evolve_step = 0n
function evolve() {
    evolve_step += 1n
    let measures = progenitor.measure_hive(evolve_step)
    console.log('measure_hive', evolve_step, progenitor.measure_hive(evolve_step))
    let solution = { seed: evolve_step, measures }
    let bin = archive_bin(solution)
    if (!map_bins[bin]) {
        map_bins[bin] = solution
    }
    setTimeout(evolve, 300)
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
    {#each {length: archive_rows} as _, row}
        <div class="row">
            {#each {length: archive_cols} as _, col}
                {@const bin = map_bins[col*archive_rows + row]}
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
