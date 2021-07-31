import { World, get_size, Snapshots } from "progenitor"
const gridSize = get_size()

export interface CellInfo {
    cell_type: number,
    value1: number,
    heading: number,
    particle: boolean,
}

export default class Simulation {
    private w = new World()
    private step: number
    private snapshots = []

    constructor () {
        this.set_rules('demo1')
    }

    set_rules(name) {
        let fname = 'set_rules_' + (name || 'demo1')
        console.log('rule selected:', fname)
        this.w = new World()  // clear rules
        this.w[fname]()
    }

    tick() {
        this.snapshots = [...this.snapshots.slice(-100), this.w.export_snapshot()]
        this.w.tick()
        this.step += 1
    }

    tick_undo() {
        if (this.snapshots.length == 0) return;
        this.w.import_snapshot(this.snapshots.pop())
        this.step -= 1
    }

    get_cell_info(col: number, row: number): CellInfo {
        return this.w.get_cell_info(col, row)
    }

    get_step() {
        return this.step
    }

    async reset() {
        for (let y = 0; y < gridSize; y++) {
            for (let x = 0; x < gridSize; x++) {
                this.w.set_cell(y, x, 0)
            }
        }
        this.w.set_cell(gridSize / 2, gridSize / 2, 1)
        this.w.set_cell(gridSize / 2 + 3, gridSize / 2 - 0, 1)
        this.w.set_cell(gridSize / 2 + 1, gridSize / 2 - 8, 1)
        this.w.set_cell(gridSize / 2 + 3, gridSize / 2 - 2, 1)
        this.step = 0
        this.snapshots = []

        // const sshot = await fetch('assets/output_0.2.dat')  /////
        const sshot = await fetch('assets/output/map_bins.dat')
        if (sshot.status !== 200) throw sshot;
        const data = new Uint8Array(await sshot.arrayBuffer())
        const snapshots = new Snapshots(data);
        console.log("snapshots", snapshots.len())
        // console.log("snapshots.getall()", snapshots.getall())

        const map = [];
        for (let i = 0; i < snapshots.len(); i++) {
            map.push({
                bin: [...snapshots.get_bin(i)],
                data: snapshots.get_data(i),
            })
        }
        // console.log("snapshots map:", map)
        const i = Math.floor(Math.random() * map.length)
        console.log('showing', map[i].bin)
        this.w.import_snapshot(map[i].data)
    }

    get_data(): Uint8Array[] {
        // contract: use it immediately, may be invalid the next time any wasm is called
        return [this.w.get_data(0), this.w.get_data(1), this.w.get_data(2)]
    }
}
