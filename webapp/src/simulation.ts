import { is_debug_build, get_size, demo_simple, demo_progenitor, demo_blobs, demo_map } from "progenitor"
import type { Simulation as ProgenitorSimulation } from 'progenitor'
const gridSize = get_size()

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

function with_three_cells(sim: ProgenitorSimulation): ProgenitorSimulation {
    sim.set_cell(gridSize / 2, gridSize / 2, 1)
    sim.set_cell(gridSize / 2 + 3, gridSize / 2 - 0, 1)
    sim.set_cell(gridSize / 2 + 1, gridSize / 2 - 8, 1)
    sim.set_cell(gridSize / 2 + 3, gridSize / 2 - 2, 1)
    return sim
}

export const rules: Rule[] = [{
    label: '1 - simple pattern',
    create: () => with_three_cells(demo_simple()),
}, {
    label: '2 - progenitor cells',
    create: () => with_three_cells(demo_progenitor()),
}, {
    label: '3 - noisy blobs',
    create: () => with_three_cells(demo_blobs()),
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
    private step: number = 0
    private snapshots = []
    private snapshot0 = this.sim.export_snapshot()

    set_rules(ruleIdx: number) {
        let rule = rules[ruleIdx]
        console.log('rule:', rule.label)
        this.sim = rule.create()
    }

    tick() {
        this.snapshots = [...this.snapshots.slice(-100), this.sim.export_snapshot()]
        this.sim.tick()
        this.step += 1
    }

    tick_undo() {
        if (this.snapshots.length == 0) return
        this.sim.import_snapshot(this.snapshots.pop())
        this.step -= 1
    }

    get_cell_info(col: number, row: number): CellInfo {
        return this.sim.get_cell_info(col, row)
    }

    get_step() {
        return this.step
    }

    reset() {
        console.log(`Simulation reset from step {this.step} to step 0.`);
        this.sim.import_snapshot(this.snapshot0)
        this.step = 0
        this.snapshots = []
    }

    get_data(): Uint8Array[] {
        // contract: use it immediately, may be invalid the next time any wasm is called
        return [this.sim.get_data(0), this.sim.get_data(1), this.sim.get_data(2)]
    }
}
