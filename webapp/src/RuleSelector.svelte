<script lang="ts">
    import { rules, Rule } from './simulation'
    export let rule: Rule
</script>

<div>
    <select bind:value={rule}>
        {#each rules as r}
            <option value={r}>{r.label}</option>
        {/each}
    </select>
</div>
