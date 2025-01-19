<script lang="ts">
import type { Rule } from './simulation'
import * as progenitor from 'progenitor'
import { onDestroy, onMount } from 'svelte'
import { type Archive, archive_cols, archive_rows, extend_archive, type Solution } from './archive'

let map_bins: Archive = $state(Array(archive_rows * archive_cols).fill(null))

let {
    selectHandler = () => {},
}: {
    selectHandler: (rule: Rule) => void
} = $props()

$effect(() => {
})

const workers: Worker[] = []

let total_evals = $state(0)
function onWorkerMessage(this: Worker, ev: MessageEvent<Archive | null>) {
    if (ev.data) {
        total_evals += 10
        extend_archive(map_bins, ev.data)
    } else {
        this.postMessage(null)  // post a second one so it doesn't wait for us to sync?
    }
    if (total_evals < 10_000) {
        // workers[0].postMessage($state.snapshot(map_bins))
        this.postMessage(null)
    }
}

onMount(() => {
    const threads = navigator.hardwareConcurrency;
    console.log('starting', threads, 'worker threads');
    for (let thread = 0; thread < threads; thread++) {
        let worker = new Worker(new URL('./worker.ts', import.meta.url), { type: 'module'})
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
<div>
    Total evals: {total_evals.toLocaleString()}  (worker threads: {workers.length})
</div>
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
