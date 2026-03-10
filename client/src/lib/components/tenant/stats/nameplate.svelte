<script lang="ts">
	let {
		channel,
		color,
		image,
	}: { channel: string; color: string; image: string } = $props();
</script>

<div class="inline md:pb-1">
	<div class="w-full shrink-0">
		<img
			src={image}
			alt={`${channel}`}
			class="rounded-full size-18 md:size-24 lg:size-32 p-1 mb-4"
		/>
	</div>
	<div
		class="mx-0.5 min-w-[99%] text-4xl font-bold lg:text-7xl"
		style:color
	>
		{channel}
	</div>
</div>
