<script lang="ts">
	import WebComponentIcon from "$lib/components/misc/icons/icon.svelte";
</script>

{#snippet LinkOpt({
	name,
	icon,
	href,
}: {
	name: string;
	icon: string;
	href: string;
})}
	<a
		{href}
		class="flex flex-row items-center space-x-4 transition-all duration-100 ease-in-out
        hover:opacity-70"
		rel="noopener noreferrer"
		target="_blank"
	>
		<div class="text-lg">{name}</div>
		<WebComponentIcon {icon} color="currentColor" size={28} />
	</a>
{/snippet}

<div class="flex w-full flex-col items-end space-y-1 self-center">
	{@render LinkOpt({
		name: "twitch",
		icon: "fa7-brands:twitch",
		href: "https://twitch.tv/plss",
	})}
	{@render LinkOpt({
		name: "github",
		icon: "fa7-brands:github",
		href: "https://github.com/plsuwu/pea-fan",
	})}
	<div class="text-muted-foreground/85 text-sm pt-3">not good at graphic design</div>
</div>
