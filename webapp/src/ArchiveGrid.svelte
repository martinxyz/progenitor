<script lang="ts">
let canvasContainer = $state<HTMLElement | undefined>(undefined)
</script>

<div>Hello, World</div>
<div class="canvasContainer" bind:this={canvasContainer}></div>
