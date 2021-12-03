<script lang="ts">
    import Simulation from './Simulation.svelte'
    import RuleSelector from './RuleSelector.svelte'
    import Container from './Container.svelte'
    import MapSelector from './MapSelector.svelte'

    let rule
    let sim: Simulation
</script>

<div class="main">
    <div>
        <Container>
            <MapSelector {sim}/>
        </Container>
        <Container>
            <RuleSelector bind:rule/>
        </Container>
        <Container>
            <Simulation bind:this={sim} {rule}/>
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
