<script lang="ts">
	let {
		name,
		isLive = false,
	}: {
		name: string;
		isLive?: boolean;
	} = $props();
</script>

<div
	class="flex w-full flex-col items-start justify-center text-lg font-bold
    sm:flex-row sm:items-center sm:text-2xl"
>
	<div class="flex sm:hidden">
		{#if name.length > 18}
			<span class="text-[13px] whitespace-nowrap">{name}</span>
		{:else if name.length > 17}
			<span class="text-sm whitespace-nowrap">{name}</span>
		{:else if name.length > 15}
			<span class="text-base whitespace-nowrap">{name}</span>
		{:else}
			<span class="whitespace-nowrap">{name}</span>
		{/if}
	</div>
	<div
		class="hidden shrink-0 flex-row items-center space-x-2 sm:flex sm:w-full"
	>
		<div class="w-full whitespace-nowrap">{name}</div>
	</div>

	{#if isLive}
		<div
			class="absolute -top-2.5 left-[88px] z-1000 shrink animate-pulse text-xs text-red-600
            uppercase sm:top-[7px] sm:left-[310px] sm:text-base md:top-[14px] md:left-90 md:text-lg
            lg:top-[17px]"
		>
			live
		</div>
	{/if}
</div>
