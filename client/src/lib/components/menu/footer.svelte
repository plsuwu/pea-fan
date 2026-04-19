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
		<div class="text-base">{name}</div>
		<WebComponentIcon {icon} color="currentColor" size={22} />
	</a>
{/snippet}

<div class="z-20 flex w-full flex-row items-center">
	<div
		class="flex w-full flex-col items-start space-y-1 self-center bg-background"
	>
		<div class="py-3 text-sm text-muted-foreground/85">contact:</div>
		<div>
			<a
				href="mailto:admin@piss.fan"
				class="flex flex-row items-center space-x-4 transition-all duration-100 ease-in-out
        hover:opacity-70"
				rel="noopener noreferrer"
				target="_blank"
			>
				<WebComponentIcon icon="lucide:mail" color="currentColor" size={22} />
				<div class="text-base underline">admin@piss.fan</div>
			</a>
		</div>
	</div>
	<div
		class="mt-4 flex w-full flex-col items-end space-y-1 self-center bg-background"
	>
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
		{@render LinkOpt({
			name: "twitter",
			icon: "fa7-brands:x-twitter",
			href: "https://x.com/owoplsuwu",
		})}
		<div class="pt-3 text-sm text-muted-foreground/85">
			not good at graphic design
		</div>
	</div>
</div>
