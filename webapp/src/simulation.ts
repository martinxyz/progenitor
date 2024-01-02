import * as progenitor from "progenitor";
import type { Simulation as ProgenitorSimulation } from 'progenitor'
import type { Viewport } from 'progenitor'

import map_bins_url from '../assets/output/map_bins.dat?url';
import turing_bins_url from '../assets/output/turing_bins.dat?url';

// Required to see rust panic message and backtrace on JS console.
// (Without it we only get the JS backtrace, saying "unreachable executed".)
progenitor.set_panic_hook()

if (progenitor.is_debug_build()) {
    console.warn("the rust wasm module was built in debug mode and will run ~100x slower")
}

export interface CellInfo {
    cell_type: number,
    value1: number,
    direction: number,
    particle: boolean,
}

export interface Rule {
    label: string,
    create: () => ProgenitorSimulation,
    load_map?: string,
    default?: boolean,
}

export const rules: Rule[] = [{
    label: 'progenitor cells',
    create: () => progenitor.demo_progenitor(),
}, {
    label: 'experiment results (select from map)',
    create: () => progenitor.demo_map(),
    load_map: map_bins_url,
}, {
    label: 'falling sand (CA)',
    create: () => progenitor.demo_falling_sand(),
}, {
    label: 'turing drawings (select from map)',
    create: () => progenitor.demo_turing(),
    load_map: turing_bins_url,
}, {
    label: 'tumblers',
    create: () => progenitor.demo_tumblers(),
}, {
    label: 'builders (random nn)',
    create: () => progenitor.demo_builders_random(),
}, {
    label: 'builders (optimized nn)',
    create: () => progenitor.demo_builders_optimized(),
}, {
    label: 'sunburn',
    create: () => progenitor.demo_sunburn(),
    default: true,
}, {
    label: 'pairs',
    create: () => progenitor.demo_pairs(),
}]

const steps_between_snapshots = 500

export default class Simulation {
    constructor(private rule: Rule) {
        this.restart()
    }
    private sim: ProgenitorSimulation
    private step_no: number
    private snapshot1: [number, Uint8Array]
    private snapshot2: [number, Uint8Array]

    restart() {
        console.log('new simulation');
        this.sim = this.rule.create()
        this.step_no = 0
        this.snapshot1 = [0, this.sim.export_snapshot()]
        this.snapshot2 = this.snapshot1
    }

    steps(count: number) {
        count = Math.round(count)
        this.sim.steps(count)
        this.step_no += count
        if (this.step_no >= this.snapshot2[0] + count * steps_between_snapshots) {
            this.snapshot1 = this.snapshot2
            this.snapshot2 = [this.step_no, this.sim.export_snapshot()]
        }
    }

    step_undo() {
        let step = this.step_no - 1
        let snapshot = this.snapshot1
        if (this.snapshot2[0] <= step) snapshot = this.snapshot2
        let count = step - snapshot[0]
        if (count < 0) return
        console.log(`replaying ${count} steps`)

        this.sim.import_snapshot(snapshot[1])
        this.sim.steps(count)
        this.step_no = step
    }

    get_cell_info(col: number, row: number): CellInfo {
        return this.sim.cell_info(col, row)
    }

    get_cell_text(col: number, row: number): string {
        return this.sim.cell_text(col, row)
    }

    get_step_no() {
        return this.step_no
    }

    get_data_viewport(): Viewport {
        return this.sim.viewport_hint()
    }

    get_data(viewport: Viewport, channel: number): Uint8Array {
        // contract: use it immediately, may be invalid the next time any wasm is called
        return this.sim.data(viewport, channel)
    }
}
