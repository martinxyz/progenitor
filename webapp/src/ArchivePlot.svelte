<script lang="ts">
import { onMount } from 'svelte'
import embed, { type VisualizationSpec } from 'vega-embed'

let { data } = $props()

/*
onMount(() => {
    if (!data) data = [{ bc1: 2, bc2: 2 }, { bc1: 5, bc2: 5 }]
    console.log('data', data)
    // if (!data) return
    spec.data = { values: data }
    embed('#vis', spec)
})
*/
$effect(() => {
    if (!data.length)
        data = [
            { bc1: 2, bc2: 2 },
            { bc1: 5, bc2: 5 },
        ]
    spec.data = { values: data }
    embed('#vis', spec)
})

const spec: VisualizationSpec = {
    $schema: 'https://vega.github.io/schema/vega-lite/v5.json',
    description: 'test plot',
    width: 800,
    height: 400,
    mark: 'circle',
    encoding: {
        x: { field: 'bc1', type: 'quantitative' },
        y: { field: 'bc2', type: 'quantitative' },
        // color: { field: 'generation', type: 'quantitative' },
        color: { field: 'fitness', type: 'quantitative' },
    },
}
</script>

<div id="vis"></div>
