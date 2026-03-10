<script lang="ts">
	let { total }: { total: number } = $props();
</script>

<div class="flex cursor-default flex-col">
	<span class="text-sm font-bold md:text-2xl">
		{total.toLocaleString()}
	</span>
	<span class="text-xs md:text-base">mentions</span>
</div>
