<script lang="ts">
	import Broadcast from './icons/Broadcast.svelte';

	let { isLive, name } = $props();
</script>

{#if isLive}
	<div class="font-display group z-[10] flex flex-row items-center">
		<div class="-mr-px mb-4 h-[5px] w-[5px] animate-ping rounded-full bg-red-600"></div>
		<Broadcast size={18} />
		<span
			class="group-hover:opacity-100 py-px flex flex-row items-center align-middle translate-x-8 absolute w-max rounded-br-md rounded-tr-lg border pl-1 pr-1 text-center text-xs opacity-0 transition-all delay-150 duration-150 ease-out"
		>
			<div>{name} is live :3</div></span
		>
        </div>
{/if}
