<script lang="ts">
import { onDestroy, onMount } from 'svelte'
import embed, { type Result, type VisualizationSpec } from 'vega-embed'

let { data } = $props()

let plot1: Result | undefined = $state()
let plot2: Result | undefined = $state()

onMount(async () => {
    plot1 = await embed('#vis1', spec1, { actions: false })
    // console.log(view.getState().signals)
    // view.addSignalListener('myhovering', (name, value) => {
    //     console.log(value)  // an array (of selected values) for each value (only with encodings: ['x', 'y'] or similar)
    // })
    // view.addSignalListener('myhovering_tuple', console.log)
    plot1.view.addEventListener('click', (event, item) => {
        let datum: any = item?.datum?.datum
        console.log(datum)
    })

    plot2 = await embed('#vis2', spec2, { actions: false })
})

onDestroy(() => {
    plot1?.finalize()
    plot2?.finalize()
})

$effect(() => {
    if (plot1) {
        plot1.view.data('mydata', $state.snapshot(data) ?? [])
        plot1.view.run()
        plot1.view.resize() // fixes missing axes after tick layout
    }
    if (plot2) {
        plot2.view.data('mydata', $state.snapshot(data) ?? [])
        plot2.view.run()
        plot2.view.resize() // fixes missing axes after tick layout
    }
})

const spec_common: Partial<VisualizationSpec> = {
    width: 'container',
    height: 400,
    background: '#ccc',
    view: {
        fill: 'white',
        // fill: null
    },
}

const spec1: VisualizationSpec = {
    ...spec_common,
    data: { name: 'mydata' },
    // description: 'test plot',
    /*
    params: [{
        name: 'grid',
        select: 'interval',
        bind: 'scales',
    }],
*/

    // "background": "orange",
    // "view": {"fill": "yellow"},
    params: [
        {
            name: 'myhovering',
            select: {
                type: 'point',
                on: 'pointerover',
                toggle: false,
                nearest: true,
                // encodings: ['x', 'y'],
            },
        },
    ],
    /*
    signals: [{
        name: 'barClick',
        value: 0,
        on: [{events: 'rect:mousedown', update: 'datum'}]
    }],
*/
    mark: 'circle',
    encoding: {
        x: { field: 'bc1', type: 'quantitative' },
        y: { field: 'bc2', type: 'quantitative' },
        color: {
            field: 'fitness',
            type: 'quantitative',
            condition: { param: 'myhovering', value: 'black', empty: false },
        },
        size: {
            condition: { param: 'myhovering', value: 100, empty: false },
            value: 20,
        },
    },
}

const spec2: VisualizationSpec = {
    ...spec_common,
    data: { name: 'mydata' },
    // description: 'test plot',

    mark: 'circle',
    encoding: {
        x: { field: 'bc1n', type: 'quantitative' },
        y: { field: 'bc2n', type: 'quantitative' },
        // color: { field: 'generation', type: 'quantitative' },
        color: { field: 'generation', type: 'nominal' },
    },
}
</script>

<details>
    <summary class="button-like">Archive (raw BCs) with fitness</summary>
    <div class="graph-container" id="vis1"></div>
</details>
<details>
    <summary class="button-like">Archive (norm BCs) with generation</summary>
    <div class="graph-container" id="vis2"></div>
</details>

<style lang="css">
.graph-container {
    width: 100%;
}
summary {
    cursor: pointer;
    user-select: none;
    border-radius: 5px;
    padding-left: 0.5em;
}
summary:hover {
    background-color: #ddd;
}
</style>
