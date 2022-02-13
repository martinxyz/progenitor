import { get_size, Snapshots, demo_simple, demo_progenitor, demo_blobs, demo_map } from "progenitor"
import { prevent_default, prop_dev } from "svelte/internal";
const gridSize = get_size()

export interface CellInfo {
    cell_type: number,
    value1: number,
    heading: number,
    particle: boolean,
}

export interface Rule {
    label: string,
    create: () => any
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
    label: '4 - experiment results (click map above)',
    create: () => demo_map(),
}];
export const default_rule_idx = 1

export default class Simulation {
    private sim = demo_simple()
    private step: number
    private snapshots = []

    set_rules(ruleIdx: number) {
        let rule = rules[ruleIdx]
        console.log('rule:', rule.label)
        this.sim = rule.create()
    }

    load_state(state: Uint8Array) {
        this.reset()
        this.sim.import_snapshot(state)
    }

    tick() {
        this.snapshots = [...this.snapshots.slice(-100), this.sim.export_snapshot()]
        this.sim.tick()
        this.step += 1
    }

    tick_undo() {
        if (this.snapshots.length == 0) return;
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
        for (let y = 0; y < gridSize; y++) {
            for (let x = 0; x < gridSize; x++) {
                this.sim.set_cell(y, x, 0)
            }
        }
        this.sim.set_cell(gridSize / 2, gridSize / 2, 1)
        this.sim.set_cell(gridSize / 2 + 3, gridSize / 2 - 0, 1)
        this.sim.set_cell(gridSize / 2 + 1, gridSize / 2 - 8, 1)
        this.sim.set_cell(gridSize / 2 + 3, gridSize / 2 - 2, 1)
        this.step = 0
        this.snapshots = []
    }

    get_data(): Uint8Array[] {
        // contract: use it immediately, may be invalid the next time any wasm is called
        return [this.sim.get_data(0), this.sim.get_data(1), this.sim.get_data(2)]
    }
}
