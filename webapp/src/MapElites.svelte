<script lang="ts">
import type { Rule } from './simulation'
import { onDestroy, onMount } from 'svelte'
import * as progenitor from 'progenitor'
import {
    type Archive,
    archive_bin,
    archive_cols,
    archive_rows,
    type Genotype,
    novelty_search_reduce,
    type Solution,
} from './archive'
import ArchivePlot from './ArchivePlot.svelte'
import type { WorkItem, WorkResult } from './worker'

let map_bins: Archive = $state(Array(archive_rows * archive_cols).fill(null))
let population: Solution[] = []
let offspring: Solution[] = []
const offspring_size = 1000
const population_size = offspring_size
const batch_size = 50

let plotData: any[] = $state([])

let {
    selectHandler = () => {},
}: {
    selectHandler: (rule: Rule) => void
} = $props()

$effect(() => {})

const workers: Worker[] = []

function initialDistribution() {
    // sample from initial distribution
    return [BigInt(Math.floor(Math.random() * 2 ** 32))]
}

function mutated(genotype: Genotype): Genotype {
    return [...genotype, BigInt(Math.floor(Math.random() * 2 ** 32))]
}

let total_evals = $state(0)
let last_perf: any
let evals_per_second: number | null = $state(null)
function onWorkerMessage(this: Worker, ev: MessageEvent<WorkResult[] | null>) {
    if (ev.data) {
        for (const result of ev.data) {
            let solution = result.solution
            let bin = archive_bin(solution)
            if (!map_bins[bin] || map_bins[bin].fitness < solution.fitness) {
                map_bins[bin] = solution
            }
            offspring.push(solution)
            if (offspring.length >= offspring_size) {
                // generation complete!
                let old_population = new Set(population)
                population = novelty_search_reduce(
                    [...population, ...offspring],
                    population_size,
                )

                plotData = population.map((s) => ({
                    bc1: s.measures_raw[0],
                    bc2: -s.measures_raw[1],
                    bc1n: s.measures_norm[0],
                    bc2n: -s.measures_norm[1],
                    fitness: s.competitionFitness,
                    generation: old_population.has(s) ? 'old' : 'new',
                }))
                offspring = []
            }
        }
    } else {
        // post a second one so it doesn't wait for us to sync? (doesn't seem to help)
        // this.postMessage([])
    }
    if (total_evals < 10_000_000) {
        // workers[0].postMessage($state.snapshot(map_bins))
        let batch: WorkItem[] = []
        for (let i = 0; i < batch_size; i++) {
            let seeds: Genotype
            let generation: number
            if (population.length < 10) {
                seeds = initialDistribution()
                generation = 0
            } else {
                // bias towards parents with high score (tournament?)
                let parent =
                    population[Math.floor(Math.random() * population.length)]
                for (let j = 0; j < 3; j++) {
                    let parent2 =
                        population[
                            Math.floor(Math.random() * population.length)
                        ]
                    parent =
                        parent.competitionFitness! > parent2.competitionFitness!
                            ? parent
                            : parent2
                }
                if (parent.generation === 0 && Math.random() < 0.25) {
                    // generation 0 has not been out-competed yet, so the initial distribution may still find something
                    seeds = initialDistribution()
                    generation = 0
                } else {
                    generation = parent.generation + 1
                    seeds = mutated(parent.seeds)
                }
            }
            batch.push({ seeds, generation })
        }
        this.postMessage(batch)
        total_evals += batch.length
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
    let rule2: Rule = {
        label: '(selected from map)',
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
    coverage: {map_bins.filter((bin) => bin != null).length}
</div>

<ArchivePlot data={plotData}></ArchivePlot>

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
                        title={bin
                            ? bin.measures_raw[0].toFixed(3) +
                              ' / ' +
                              bin.measures_raw[1].toFixed(3) +
                              ` (gen ${bin.generation})`
                            : 'empty'}
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
