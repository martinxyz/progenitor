<div class="table">
    {#each bins_rows as row}
        <div class="row">
            {#each bins_cols as col}
                <div class="cell">
                    <div class="inner" title={`${row}, ${col}`}></div>
                </div>
            {/each}
        </div>
    {/each}
</div>

<style lang="scss">
    .table {
        /* background-color: solid #2e170840; */
        /* border-left: 1px solid black; */
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
        /* border: 1px solid black; */
        /* border: 2px solid #2E170E10; */
        /* background-color: black; */
        padding: 1px;
    }
    .inner {
        background-color: #2E170E80;
        width: 100%;
        height: 100%;
    }
</style>

<script lang="ts">
    import { onMount } from "svelte";
    import { Snapshots } from "progenitor";

     let bins_rows = [...Array(30).keys()]
     let bins_cols = [...Array(30).keys()]

     onMount(async () => {
         // const sshot = await fetch('assets/output_0.2.dat')  /////
         const sshot = await fetch('assets/output/map_bins.dat')
         if (sshot.status !== 200) throw sshot;
         const data = new Uint8Array(await sshot.arrayBuffer())
         const snapshots = new Snapshots(data);
         console.log("snapshots", snapshots.len())
         console.log("snapshots.getall()", snapshots.getall())

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
         // this.w.import_snapshot(map[i].data)
     })
</script>
