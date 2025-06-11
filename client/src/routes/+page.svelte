<script lang="ts">
	import type { PageProps } from './$types';
	let { data }: PageProps = $props();

	console.log(data);
</script>

<div>
	{#each data.fans as fan}
		<div>{fan}</div>
	{/each}
</div>
