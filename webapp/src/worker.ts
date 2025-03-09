import * as progenitor from 'progenitor'
import { type Solution } from './archive'

import './progenitor_wasm_init'

function evolve(seeds: bigint[]): Solution {
    let measures = progenitor.measure_rainfall(new BigUint64Array(seeds))
    return { seeds, measures }
}

onmessage = function (event: MessageEvent<bigint[][]>) {
    let solutions: Solution[] = []
    for (let seed of event.data) {
        solutions.push(evolve(seed))
    }
    postMessage(solutions)
}

postMessage(null) // ready signal to main thread (onmessage handler is bound)
