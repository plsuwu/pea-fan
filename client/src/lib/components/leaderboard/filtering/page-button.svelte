<script lang="ts">
	import { cn } from "$lib/shadcn-components/utils";
	import type { Component } from "svelte";
	import Diamond from "$lib/components/misc/diamond.svelte";
	import { goto, pushState } from "$app/navigation";
	import { page } from "$app/state";

	let {
		icon: Icon,
		href,
		disabled = false,
		direction,
		class: className = "",
	}: {
		icon: Component;
		href: string;
		disabled?: boolean;
		direction: "forward" | "backward";
		class?: string;
	} = $props();

	const navigateNoScroll = (url: string) => {
		goto(url, { noScroll: true });
	};

	const hoverShift = $derived.by(() =>
		direction === "forward"
			? "group-hover:translate-x-0.5 group-active:translate-x-1"
			: "group-hover:-translate-x-0.5 group-active:-translate-x-1"
	);

	const iconClass = $derived(
		cn(
			"absolute transition-all duration-100",
			disabled ? "text-muted-foreground/60" : hoverShift,
			className
		)
	);

	const diamondClass = $derived(
		cn(
			"absolute bg-transparent transition-all duration-100",
			!disabled && hoverShift
		)
	);
</script>

{#if disabled}
	<div class="group flex cursor-not-allowed items-center justify-center p-2">
		<Icon class={iconClass} />
		<div class={diamondClass}>
			<Diamond size={40} {disabled} />
		</div>
	</div>
{:else}
	<button
		onclick={() => navigateNoScroll(href)}
		class="group flex items-center justify-center p-2"
	>
		<Icon class={iconClass} />
		<div class={diamondClass}>
			<Diamond size={40} {disabled} />
		</div>
	</button>
{/if}
