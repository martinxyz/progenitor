import * as progenitor from 'progenitor'
import { type Genotype, measures_normalized, type Solution } from './archive'

import './progenitor_wasm_init'

export type WorkItem = {
    seeds: Genotype
    generation: number
}

export type WorkResult = {
    solution: Solution
}

function evolve(seeds: bigint[], generation: number): Solution {
    let measures_raw = progenitor.measure_rainfall(new BigUint64Array(seeds))
    let fitness = 0

    if (!isFinite(fitness ?? NaN)) {
        throw new Error('fitness is not finite')
    }
    if (!measures_raw.every(isFinite)) {
        console.log(measures_raw)
        throw new Error('measures are not finite')
    }
    let measures_norm = measures_normalized(measures_raw)
    return { seeds, measures_raw, measures_norm, fitness, generation }
}

onmessage = function (event: MessageEvent<WorkItem[]>) {
    let results: WorkResult[] = []
    for (let item of event.data) {
        let solution = evolve(item.seeds, item.generation)
        results.push({ solution })
    }
    postMessage(results)
}

postMessage(null) // ready signal to main thread (onmessage handler is bound)
