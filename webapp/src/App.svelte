<script lang="ts">
    import SimulationPlayer from './SimulationPlayer.svelte'
    import RuleSelector from './RuleSelector.svelte'
    import Container from './Container.svelte'
    import MapSelector from './MapSelector.svelte'
    import Simulation, { rules } from './simulation'
    import type { Rule } from './simulation'

    let rule: Rule = rules.find(rule => rule.default)

    let sim: Simulation

    $: sim = new Simulation(rule)

    function onMapSelected(derivedRule: Rule) {
        sim = new Simulation(derivedRule)
    }
</script>

<div class="main">
    <div>
        <Container>
            <div class="row">
                <RuleSelector bind:rule/>
                <a href="https://github.com/martinxyz/progenitor">{'{src}'}</a>
            </div>
        </Container>
        {#if rule.load_map}
            <Container>
                <MapSelector selectHandler={onMapSelected} {rule} />
            </Container>
        {/if}
        <Container>
            <SimulationPlayer {sim} />
        </Container>
    </div>
</div>

<style lang="scss">
    .main {
        padding: 1em;
        display: flex;
        justify-content: center;
    }
    .row {
        display: flex;
        align-items: center;
        justify-content: space-between;
    }
    a {
        padding: 0 1em;
    }
</style>
