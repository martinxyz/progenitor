<script lang="ts">
    import SimulationPlayer from './SimulationPlayer.svelte'
    import RuleSelector from './RuleSelector.svelte'
    import Container from './Container.svelte'
    import MapSelector from './MapSelector.svelte'
    import Simulation, { default_rule_idx, Rule, rules } from './simulation'

    let rule: Rule = rules[default_rule_idx]
    let sim: Simulation

    $: sim = new Simulation(rule)

    function onMapSelected(derivedRule: Rule) {
        sim = new Simulation(derivedRule)
    }
</script>

<div class="main">
    <div>
        <Container>
            <RuleSelector bind:rule/>
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
</style>
