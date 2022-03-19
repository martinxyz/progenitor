import { is_debug_build, demo_simple, demo_progenitor, demo_blobs, demo_map, set_panic_hook } from "progenitor"
import type { Simulation as ProgenitorSimulation } from 'progenitor'

// Required to see rust panic message and backtrace on JS console.
// (Without it we only get the JS backtrace, saying "unreachable executed".)
set_panic_hook()

if (is_debug_build()) {
    console.warn("the rust wasm module was built in debug mode and will run ~100x slower")
}

export interface CellInfo {
    cell_type: number,
    value1: number,
    heading: number,
    particle: boolean,
}

export interface Rule {
    label: string,
    create: () => any,
    show_map?: boolean,
}

export const rules: Rule[] = [{
    label: '1 - simple pattern',
    create: () => demo_simple(),
}, {
    label: '2 - progenitor cells',
    create: () => demo_progenitor(),
}, {
    label: '3 - noisy blobs',
    create: () => demo_blobs(),
}, {
    label: '4 - experiment results (select from map)',
    create: () => demo_map(),
    show_map: true,
}];
export const default_rule_idx = 1

export default class Simulation {
    constructor(private sim: ProgenitorSimulation) {
        console.log('Simulation constructor')
    }
    private step_no: number = 0
    private snapshots = []
    private snapshot0 = this.sim.export_snapshot()

    step() {
        this.snapshots = [...this.snapshots.slice(-100), this.sim.export_snapshot()]
        this.sim.step()
        this.step_no += 1
    }

    step_undo() {
        if (this.snapshots.length == 0) return
        this.sim.import_snapshot(this.snapshots.pop())
        this.step_no -= 1
    }

    get_cell_info(col: number, row: number): CellInfo {
        return this.sim.get_cell_info(col, row)
    }

    get_step_no() {
        return this.step_no
    }

    reset() {
        console.log(`Simulation reset from step {this.step_no} to step 0.`);
        this.sim.import_snapshot(this.snapshot0)
        this.step_no = 0
        this.snapshots = []
    }

    get_data(): Uint8Array[] {
        // contract: use it immediately, may be invalid the next time any wasm is called
        return [this.sim.get_data(0), this.sim.get_data(1), this.sim.get_data(2)]
    }
}
