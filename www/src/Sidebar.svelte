<script lang="ts">
    import type { CellInfo } from './simulation';
    export let cell: CellInfo | null;
</script>

<div class="infobar">
    <h3>Info</h3>
    {#if cell}
        CellType { cell.cell_type }
        <pre>{JSON.stringify(cell, null, 2)}</pre>
    {:else}
        Select a cell.
    {/if}
</div>

<style>
 .infobar {
     padding-left: 1em;
     min-width: 16em;
 }
</style>
