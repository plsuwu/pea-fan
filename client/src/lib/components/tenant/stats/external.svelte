<script lang="ts">
	import LinkJson from "./links.json";
	import WebComponentIcon from "$lib/components/misc/icons/icon.svelte";

	const DEFAULT_ICON_SIZE = 19;

	let { channelLogin, channelId }: { channelLogin: string; channelId: string } =
		$props();

	export const LINK_ICON_NAME_MAP: Record<
		string,
		{ icon: string; color: string; size?: number }
	> = {
		x: { icon: "fa7-brands:x-twitter", color: "currentColor" },
		twitter: { icon: "fa7-brands:x-twitter", color: "currentColor" },
		bsky: { icon: "fa7-brands:bluesky", color: "#0A7AFF" },
		youtube: { icon: "fa7-brands:youtube", color: "#CD201F" },
		patreon: { icon: "fa7-brands:patreon", color: "#F96854" },
		discord: { icon: "fa7-brands:discord", color: "#5865F2" },
		tiktok: {
			icon: "fa7-brands:tiktok",
			color: "currentColor",
			size: DEFAULT_ICON_SIZE - 2,
		},
		instagram: { icon: "fa7-brands:instagram", color: "currentColor" },
		github: { icon: "fa7-brands:github", color: "currentColor" },
	} as const;

	let externals = $derived.by(() => {
		const channelEntry =
			LinkJson.find((entry) => entry.id === channelId) || null;

		if (channelEntry) {
			return channelEntry.data.links.map((link) => {
				const { url, name } = link;

				let icon = LINK_ICON_NAME_MAP[name];
				if (!icon) {
					icon = { icon: "ic:round-link", color: "currentColor", size: 18 };
				}

				return { icon, url };
			});
		}
	});
</script>

{#snippet ExternalUrl(
	href: string,
	iconData: { icon: string; color: string; size?: number }
)}
	<a
		{href}
		target="_blank"
		rel="noreferrer noopener"
		class="flex items-center justify-center px-1 text-center brightness-65
        transition-discrete duration-200 ease-out hover:brightness-100"
	>
		<WebComponentIcon
			icon={iconData.icon}
			color={"currentColor"}
			size={iconData.size || DEFAULT_ICON_SIZE}
		/>
	</a>
{/snippet}

<div
	class="mt-1.5 mb-px flex place-items-center items-center justify-start px-1"
>
	{@render ExternalUrl(`https://twitch.tv/${channelLogin}`, {
		icon: "fa7-brands:twitch",
		color: "#6441a5",
	})}
	{#if externals}
		{#each externals as ext}
			{@render ExternalUrl(ext.url, ext.icon)}
		{/each}
	{/if}
</div>
