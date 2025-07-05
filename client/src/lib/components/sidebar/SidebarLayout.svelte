<script lang="ts">
	import Tabs from './options/Tabs.svelte';

    let { data } = $props();
</script>

<aside
	class="flex h-[calc(100vh-60px)] w-[350px] shrink-0 flex-col justify-between overflow-hidden border-r py-3"
>
	<Tabs {data} />
	<div class="inline-flex w-full flex-col px-4">
		<div class="text-[12px]">don't want your username displayed?</div>
		<a
			class="hover:text-foreground-alt w-fit text-[12px] italic transition-colors duration-150 ease-out underline"
			href="/redact">redact your username here.</a
		>
	</div>
</aside>
