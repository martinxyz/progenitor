<script lang="ts">
    import Simulation from './Simulation.svelte'
    import RuleSelector from './RuleSelector.svelte'
    import Container from './Container.svelte'
    import MapSelector from './MapSelector.svelte'

    let rule
</script>

<div class="main">
    <div>
        <Container>
            <MapSelector/>
        </Container>
        <Container>
            <RuleSelector bind:rule/>
        </Container>
        <Container>
            <Simulation {rule}/>
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
