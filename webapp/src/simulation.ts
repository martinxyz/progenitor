import * as progenitor from "progenitor";
import type { Simulation as ProgenitorSimulation } from 'progenitor'

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
    create: () => any,
    show_map?: boolean,
}

export const rules: Rule[] = [{
    label: '1 - simple pattern',
    create: () => progenitor.demo_simple(),
}, {
    label: '2 - progenitor cells',
    create: () => progenitor.demo_progenitor(),
}, {
    label: '3 - noisy blobs',
    create: () => progenitor.demo_blobs(),
}, {
    label: '4 - experiment results (select from map)',
    create: () => progenitor.demo_map(),
    show_map: true,
}, {
    label: '5 - sim2',
    create: () => progenitor.demo_sim2(),
}, {
    label: '6a - turing drawings 1',
    create: () => progenitor.demo_turing1(),
}, {
    label: '6b - turing drawings 2',
    create: () => progenitor.demo_turing2(),
}];

export const default_rule_idx = 6

const steps_between_snapshots = 500

export default class Simulation {
    constructor(private sim: ProgenitorSimulation) {
        console.log('Simulation constructor')
    }
    private step_no = 0
    private snapshot0: [number, Uint8Array] = [0, this.sim.export_snapshot()]
    private snapshot1: [number, Uint8Array] = this.snapshot0
    private snapshot2: [number, Uint8Array] = this.snapshot0

    step() {
        this.sim.step()
        this.step_no += 1
        if (this.step_no >= this.snapshot2[0] + steps_between_snapshots) {
            this.snapshot1 = this.snapshot2
            this.snapshot2 = [this.step_no, this.sim.export_snapshot()]
        }
    }

    step_undo() {
        let step = this.step_no - 1
        let snapshot = this.snapshot0
        if (this.snapshot1[0] <= step) snapshot = this.snapshot1
        if (this.snapshot2[0] <= step) snapshot = this.snapshot2
        let count = step - snapshot[0]
        console.log('replay steps:', count)
        if (count > steps_between_snapshots) return

        this.sim.import_snapshot(snapshot[1])
        for (let i=0; i<count; i++) this.sim.step()
        this.step_no = step
    }

    get_cell_info(col: number, row: number): CellInfo {
        return this.sim.get_cell_info(col, row)
    }

    get_cell_text(col: number, row: number): string {
        return this.sim.get_cell_text(col, row)
    }

    get_step_no() {
        return this.step_no
    }

    reset() {
        console.log(`Simulation reset from step {this.step_no} to step 0.`);
        this.sim.import_snapshot(this.snapshot0[1])
        this.step_no = 0
        this.snapshot1 = this.snapshot0
        this.snapshot2 = this.snapshot0
    }

    get_data(): Uint8Array[] {
        // contract: use it immediately, may be invalid the next time any wasm is called
        return [this.sim.get_data(0), this.sim.get_data(1), this.sim.get_data(2)]
    }
}
