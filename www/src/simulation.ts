import { World, get_size } from "progenitor"
const gridSize = get_size()

export interface CellInfo {
    cell_type: number,
    value1: number,
    value2: number,
    particle: boolean,
}

export default class Simulation {
    constructor (
        private w = new World()
    ) {
        this.w.set_rules_demo3()
    }

    tick() {
        this.w.tick()
    }

    get_cell_info(col: number, row: number): CellInfo {
        return JSON.parse(this.w.get_cell_json(col, row))
    }

    reset() {
        for (let y = 0; y < gridSize; y++) {
            for (let x = 0; x < gridSize; x++) {
                this.w.set_cell(y, x, 0)
            }
        }
        this.w.set_cell(gridSize / 2, gridSize / 2, 1)
        this.w.set_cell(gridSize / 2 + 3, gridSize / 2 - 0, 1)
        this.w.set_cell(gridSize / 2 + 1, gridSize / 2 - 8, 1)
        this.w.set_cell(gridSize / 2 + 3, gridSize / 2 - 2, 1)

    }

    update_data(): Uint8Array {
        // contract: use it immediately, may be invalid the next time any wasm is called
        return this.w.update_data()
    }
}
