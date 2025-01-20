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

function evolve() {
    let seed = BigInt(Math.floor(Math.random() * 2 ** 32))
    let measures = progenitor.measure_hive(seed)
    let solution = { seed, measures }
    let bin = archive_bin(solution)
    if (!map_bins[bin]) {
        map_bins[bin] = solution
    }
}

onmessage = function (event: MessageEvent<null>) {
    for (let i = 0; i < 10; i++) {
        evolve()
    }
    postMessage(map_bins)
}

postMessage(null) // ready signal to main thread (onmessage handler is bound)
