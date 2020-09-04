<script lang="ts">
    import Sidebar from './Sidebar.svelte'
    import Simulation from './Simulation.svelte'
</script>

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
        background-color: #CCC;
    }
</style>
