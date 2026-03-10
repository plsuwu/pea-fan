<script lang="ts">
	import WebComponentIcon from "$lib/components/misc/icons/icon.svelte";
</script>

{#snippet LinkOpt({
	name,
	icon,
	href
}: {
	name: string;
	icon: string;
	href: string;
})}
	<a
		{href}
		class="flex flex-row items-center"
		rel="noopener noreferrer"
		target="_blank"
	>
		<div>{name}</div>
		<WebComponentIcon {icon} color="currentColor" size={28} />
	</a>
{/snippet}

<div class="flex w-full flex-row justify-end">
	{@render LinkOpt({
		name: "GitHub",
		icon: "fa7-brands:github",
		href: "https://github.com/plsuwu"
	})}
</div>
