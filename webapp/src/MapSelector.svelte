<script lang="ts">
    import { onMount } from 'svelte';
    import { demo_turing, Snapshots } from 'progenitor';
    import type { Rule } from './simulation'

    let map_bins: Uint8Array[][] = []
    let bins_rows: number[] = []
    let bins_cols: number[] = []

    export let selectHandler: (rule: Rule) => void = () => {}

    onMount(async () => {
        // const sshot = await fetch('assets/output/map_bins.dat')
        const sshot = await fetch('assets/output/turing_bins.dat')
        if (sshot.status !== 200) throw sshot;
        const data = new Uint8Array(await sshot.arrayBuffer())
        const snapshots = new Snapshots(data);

        var f1_values = []
        var f2_values = []
        const map = snapshots.getall()
        for (const item of map) {
            const f1: number = item[0]
            const f2: number = item[1]
            f1_values.push(f1)
            f2_values.push(f2)
        }
        const f1_min = Math.min(...f1_values)
        const f1_max = Math.max(...f1_values)
        const f2_min = Math.min(...f2_values)
        const f2_max = Math.max(...f2_values)
        for (let i=0; i <= f1_max-f1_min; i++) bins_cols.push(i)
        for (let i=0; i <= f2_max-f2_min; i++) bins_rows.push(i)
        bins_rows.reverse()

        map_bins = []
        for (let _ of bins_rows) {
            var r = []
            map_bins.push(r)
            for (let _ of bins_cols) {
                r.push(null)
            }
        }
        for (const item of map) {
            const f1: number = item[0] - f1_min
            const f2: number = item[1] - f2_min
            const data: Uint8Array = item[2]
            map_bins[f2][f1] = data
        }
    })

    function loadbin(bin: (Uint8Array|null)) {
        if (!bin) return
        let rule: Rule = {
            label: '(selected from map)',
            create: () => {
                // let sim_rust = demo_map()
                let sim_rust = demo_turing()
                sim_rust.import_snapshot(bin)
                return sim_rust
            }
        }
        selectHandler(rule)
    }

</script>

<div class="table">
    {#each bins_rows as row}
        <div class="row">
            {#each bins_cols as col}
                <div class="cell" class:full={map_bins[row][col] != null} on:click={() => loadbin(map_bins[row][col])}>
                    <div title={`${row}, ${col}`} class="inner">
                    </div>
                </div>
            {/each}
        </div>
    {/each}
</div>

<style lang="scss">
    .table {
        width: 100%;
        height: 20em;
        display: flex;
        flex-direction: column;
    }
    .row {
        flex-grow: 1;
        display: flex;
        flex-direction: row;
    }
    .cell {
        flex-grow: 1;
        padding: 1px;
    }
    .inner {
        background-color: #2E170E80;
        width: 100%;
        height: 100%;
    }
    .full > .inner {
        background-color: #000;
    }
    .full:hover {
        cursor: pointer;
        & > .inner {
            background-color: #055;
        }
    }

</style>
