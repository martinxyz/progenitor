<script lang="ts">
    export let name: string
    import Sidebar from './Sidebar.svelte'
    import Simulation from './Simulation.svelte'
</script>

<h1>Hi {name}!</h1>
<div class="main">
    <Simulation/>
    <Sidebar/>
</div>

<style>
    .main {
        /* text-align: center; */
        padding: 1em;
        /* margin: 0 auto; */
        display: flex;
        flex-direction: row;
    }

    h1 {
        color: #ff3e00;
        font-size: 2em;
        font-weight: 100;
    }
</style>
