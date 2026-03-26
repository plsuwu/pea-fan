<script lang="ts">
	import Square from "$lib/components/misc/square.svelte";
	import { getRankColor, readableColor } from "$lib/utils";
	let { ranking }: { ranking: number } = $props();
    
	let { color, thickness } = $derived(getRankColor(ranking));
</script>

<div class="flex items-center">
	<div class="group flex items-center justify-center pl-2.5 lg:p-4">
		<div
			class="absolute text-sm font-bold lg:text-xl"
			style={`color: ${ranking <= 3 ? color : 'var(--color-foreground)'}`}
		>
			{ranking}
		</div>
		<div class="absolute bg-transparent transition-all duration-100">
			<div class="block lg:hidden">
				<Square size={25} stroke={readableColor(color)} {thickness} />
			</div>
			<div class="hidden lg:block">
				<Square size={40} stroke={readableColor(color)} {thickness} />
			</div>
		</div>
	</div>
</div>
