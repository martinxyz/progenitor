<script lang="ts">
    import type { CellInfo } from "./simulation"
    export let cellText: string | null
    export let showEnergy: boolean
    export let showDirection: boolean
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
    .text {
        white-space: pre-wrap;
    }
</style>

<div class="infobar">
    <h3>Display</h3>
    <p>
        <label class="checkbox">
            <input type="checkbox" bind:checked={showEnergy}/>
            Energy
        </label>
        <br/>
        <label class="checkbox">
            <input type="checkbox" bind:checked={showDirection}/>
            Direction
        </label>
    </p>
    <h3>Info</h3>
    {#if cellText}
        <p class="text">{cellText}</p>
    {:else}
        Select a cell.
    {/if}
</div>
