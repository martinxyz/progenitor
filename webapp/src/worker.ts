import * as progenitor from 'progenitor'
import {
    archive_bin,
    archive_cols,
    archive_rows,
    type Archive,
    type Solution,
} from './archive'

import './progenitor_wasm_init'

let map_bins: Archive = Array(archive_rows * archive_cols).fill(null)

function evolve(seeds: bigint[]) {
    let measures = progenitor.measure_hive(new BigUint64Array(seeds))
    let solution = { seeds, measures }
    let bin = archive_bin(solution)
    if (!map_bins[bin]) {
        map_bins[bin] = solution
    }
}

onmessage = function (event: MessageEvent<bigint[][]>) {
    for (let seed of event.data) {
        evolve(seed)
    }
    postMessage(map_bins)
}

postMessage(null) // ready signal to main thread (onmessage handler is bound)
