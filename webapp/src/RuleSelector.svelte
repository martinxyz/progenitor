<script>
    import { rules, default_rule_idx } from './simulation'
    export let rule = default_rule_idx
</script>

<div>
    <select bind:value={rule}>
        {#each rules as r, idx}
            <option value={idx}>{r.label}</option>
        {/each}
    </select>
</div>
