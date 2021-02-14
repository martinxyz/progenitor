<script lang="ts">
    import type { CellInfo } from "./simulation"
    export let cell: CellInfo | null
    export let showEnergy: boolean
</script>

<style lang="scss">
    .infobar {
        border-left: 1px solid #2e170840;
        margin-left: 0.8em;
        padding-left: 0.4em;
        width: 15em;
        height: 100%;
    }
    h3 {
        margin-bottom: .4em;
    }
    p {
        margin-bottom: 1.2em;
    }
</style>

<div class="infobar">
    <h3>Display</h3>
    <p>
        <label>
            <input type="checkbox" bind:checked={showEnergy}/>
            Energy
        </label>
    </p>
    <h3>Info</h3>
    {#if cell}
        CellType: {cell.cell_type}
        <!-- <br /> Value1: {cell.value1}
             <br /> Heading: {cell.heading}
             <br /> Particle: {cell.particle}
             <br /> -->
        <pre>{JSON.stringify(cell, null, 2)}</pre>
    {:else}
        Select a cell.
    {/if}
</div>
