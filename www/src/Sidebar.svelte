<script lang="ts">
    import type { CellInfo } from './simulation';
    export let cell: CellInfo | null;
</script>

<div class="infobar">
    <h3>Info</h3>
    {#if cell}
        CellType: { cell.cell_type } <br/>
        Value1: { cell.value1 } <br/>
        Value2: { cell.value2 } <br/>
        Particle: { cell.particle } <br/>
        <!-- <pre>{JSON.stringify(cell, null, 2)}</pre> -->
    {:else}
        Select a cell.
    {/if}
</div>

<style lang="scss">
    .infobar {
        border-left: 1px solid #2E170840;
        margin-left: .8em;
        padding-left: .4em;
        width: 15em;
        height: 100%;
    }
</style>
