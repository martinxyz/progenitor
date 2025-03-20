<script lang="ts">
import { onDestroy, onMount } from 'svelte'
import embed, { type Result, vega, type VisualizationSpec } from 'vega-embed'

let { data } = $props()

let plot: Result

onMount(async () => {
    // plot = await embed('#vis', spec)
})

onDestroy(() => {
    plot?.finalize()
})

$effect(() => {
    plot?.finalize()
    spec.data = { values: $state.snapshot(data) }
    embed('#vis', spec, { actions: false }).then(p => plot = p)
    // should not re-create the plot on every change...?
    // let changeset = vega.changeset().remove(() => true).insert(data);
    // plot.view.change('source_0', changeset).run()
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
