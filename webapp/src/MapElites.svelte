<script lang="ts">
import type { Rule } from './simulation'
import { onDestroy, onMount } from 'svelte'
import * as progenitor from 'progenitor'
import {
    type Archive,
    archive_cols,
    archive_rows,
    extend_archive,
    type Genotype,
    type Solution,
} from './archive'

let map_bins: Archive = $state(Array(archive_rows * archive_cols).fill(null))

let {
    selectHandler = () => {},
}: {
    selectHandler: (rule: Rule) => void
} = $props()

$effect(() => {})

const workers: Worker[] = []
// let last_seed = 0n
// let seeds = [BigInt(Math.floor(Math.random() * 2 ** 32))]

function randomArchiveEntryMutated(): Genotype {
    let candidates: (Solution | null)[] = map_bins.filter((entry) => !!entry)
    candidates.push(null)
    let entry = candidates[Math.floor(Math.random() * candidates.length)]
    if (entry === null) {
        // sample from initial distribution
        return [BigInt(Math.floor(Math.random() * 2 ** 32))]
    } else {
        return [...entry.seeds, BigInt(Math.floor(Math.random() * 2 ** 32))]
    }
}

let total_evals = $state(0)
let last_perf: any
let evals_per_second: number | null = $state(null)
function onWorkerMessage(this: Worker, ev: MessageEvent<Archive | null>) {
    if (ev.data) {
        extend_archive(map_bins, ev.data)
    } else {
        // post a second one so it doesn't wait for us to sync? (doesn't seem to help)
        // this.postMessage([])
    }
    if (total_evals < 10_000_000) {
        // workers[0].postMessage($state.snapshot(map_bins))
        let solutions = []
        for (let i = 0; i < 50; i++) {
            // last_seed += 1n
            let solution: Genotype = randomArchiveEntryMutated()
            solutions.push(solution)
        }
        this.postMessage(solutions)
        total_evals += solutions.length
        let ts = performance.now()
        if (last_perf && ts > last_perf.ts + 3_000) {
            evals_per_second =
                ((total_evals - last_perf.total_evals) / (ts - last_perf.ts)) *
                1000
            last_perf = null
        }
        if (!last_perf) {
            last_perf = { ts, total_evals }
        }
    }
}

onMount(() => {
    const threads = navigator.hardwareConcurrency
    console.log('starting', threads, 'worker threads')
    for (let thread = 0; thread < threads; thread++) {
        let worker = new Worker(new URL('./worker.ts', import.meta.url), {
            type: 'module',
        })
        workers.push(worker)
        worker.onmessage = onWorkerMessage
    }
})

onDestroy(() => {
    console.log('destroying', workers.length, 'worker threads')
    while (workers.length) {
        workers.pop()!.terminate()
    }
})

function loadbin(bin: Genotype | null) {
    if (!bin) return
    let rule2: Rule = { label: '(selected from map)',
        create: () => {
            return progenitor.demo_rainfall(new BigUint64Array(bin))
        },
    }
    selectHandler(rule2)
}
</script>

<div>
    Total evals: {total_evals.toLocaleString()} ({workers.length} threads)
    {#if evals_per_second}
        ({evals_per_second.toFixed(0)?.toLocaleString()} evals per second)
    {/if}
</div>
<div class="table">
    {#each { length: archive_rows } as _, row}
        <div class="row">
            {#each { length: archive_cols } as _, col}
                {@const bin = map_bins[col * archive_rows + row]}
                <div
                    class="cell"
                    class:full={bin != null}
                    onclick={() => {
                        if (bin) loadbin(bin.seeds)
                    }}
                >
                    <div
                        title={bin?.measures[0].toFixed(3) +
                            ' / ' +
                            bin?.measures[1].toFixed(3)}
                        class="inner"
                    ></div>
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
