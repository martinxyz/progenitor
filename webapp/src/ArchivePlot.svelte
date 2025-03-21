<script lang="ts">
import { onDestroy, onMount } from 'svelte'
import embed, { type Result, type VisualizationSpec } from 'vega-embed'

let { data } = $props()

let plot1: Result | undefined = $state()
let plot2: Result | undefined = $state()

onMount(async () => {
    plot1 = await embed('#vis1', spec1, { actions: false, })
    // console.log(view.getState().signals)
    // view.addSignalListener('myhovering', (name, value) => {
    //     console.log(value)  // an array (of selected values) for each value (only with encodings: ['x', 'y'] or similar)
    // })
    // view.addSignalListener('myhovering_tuple', console.log)
    plot1.view.addEventListener('click', (event, item) => {
        let datum: any = item?.datum?.datum
        console.log(datum)
    })

    plot2 = await embed('#vis2', spec2, { actions: false, })
})

onDestroy(() => {
    plot1?.finalize()
    plot2?.finalize()
})

$effect(() => {
    if (plot1) {
        plot1.view.data('mydata', $state.snapshot(data) ?? [])
        plot1.view.run()
    }
    if (plot2) {
        plot2.view.data('mydata', $state.snapshot(data) ?? [])
        plot2.view.run()
    }
})

const spec1: VisualizationSpec = {
    $schema: 'https://vega.github.io/schema/vega-lite/v5.json',
    description: 'test plot',
/*
    params: [{
        name: 'grid',
        select: 'interval',
        bind: 'scales',
    }],
*/
    data: {name: 'mydata'},
    params: [{
        name: 'myhovering',
        select: {
            type: 'point',
            on: 'pointerover',
            toggle: false,
            nearest: true,
            // encodings: ['x', 'y'],
        }
    }],
/*
    signals: [{
        name: 'barClick',
        value: 0,
        on: [{events: 'rect:mousedown', update: 'datum'}]
    }],
*/
    width: 'container',
    height: 400,
    mark: 'circle',
    encoding: {
        x: { field: 'bc1', type: 'quantitative' },
        y: { field: 'bc2', type: 'quantitative' },
        color: {
            field: 'fitness',
            type: 'quantitative',
            condition: {param: 'myhovering', value: 'black', empty: false},
        },
        size: {
            condition: {param: 'myhovering', value: 100, empty: false},
            value: 20,
        }
    },
}

const spec2: VisualizationSpec = {
    $schema: 'https://vega.github.io/schema/vega-lite/v5.json',
    description: 'test plot',
    width: 'container',
    height: 400,
    mark: 'circle',
    data: {name: 'mydata'},
    encoding: {
        x: { field: 'bc1n', type: 'quantitative' },
        y: { field: 'bc2n', type: 'quantitative' },
        // color: { field: 'generation', type: 'quantitative' },
        color: { field: 'generation', type: 'nominal' },
    },
}
</script>

<details open>
    <summary>Plot1</summary>
    <div class="graph-container" id="vis1"></div>
</details>
<details>
    <summary>Plot2</summary>
    <div class="graph-container" id="vis2"></div>
</details>

<style lang="css">
  .graph-container {
      width: 100%;
  }
</style>
