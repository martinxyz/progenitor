<script>
    export let rule
</script>

<div>
    <select bind:value={rule}>
        <option value={'demo1'}>1 - simple pattern</option>
        <option value={'demo2'}>2 - progrenitor cells</option>
        <option value={'demo3'}>3 - noisy blobs</option>
        <option value={'demo4'} selected>4 - experiment results (click map above)</option>
    </select>
</div>
