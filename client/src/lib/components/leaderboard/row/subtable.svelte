<script lang="ts">
	import { page } from "$app/state";
	import * as Tt from "$lib/shadcn-components/ui/tooltip/index";
	import { getAltImageSizeUrl } from "$lib/utils";
	// import { URLS } from "$lib";
	import { Rh } from "$lib/utils/route";
	import SkeletonImage from "./skeleton-image.svelte";

	let { login, totalScores, scores, variant } = $props();

	let preposition = $derived(variant === "Channel" ? "by" : "in");
	let { host } = $derived(page.url);

	const PREVIEWS_COUNT = 6;

	function getMentionedSubtitle(
		scoresCount: number,
		numPreviews = PREVIEWS_COUNT
	) {
		const remaining = scoresCount - numPreviews;
		let subtitle = `+ ${remaining.toLocaleString()}`;

		if (remaining === 1) {
			return subtitle + " other";
		}

		return subtitle + " others";
	}
</script>

{#snippet Hoverable({
	login,
	name,
	color,
	imgUrl,
	total,
}: {
	login: string;
	name: string;
	color: string;
	imgUrl: string;
	total: number;
})}
	{@const href = `https://twitch.tv/${login}`}

	<a {href} rel="noreferrer noopener" target="_blank">
		<Tt.Root>
			<Tt.Trigger>
				<SkeletonImage
					src={getAltImageSizeUrl(imgUrl, "XS")}
					alt={login}
					class="z-10 rounded-full bg-background ring-1 ring-offset-1"
					skeletonClass="size-6 rounded-full"
				/>
			</Tt.Trigger>
			<Tt.Content
				class="border border-accent-foreground bg-background font-iosevka text-foreground"
				arrowClasses="bg-background border-b border-r border-accent-foreground"
			>
				<div
					class="flex w-full justify-between text-base font-semibold"
					style={`color: ${color};`}
				>
					{name}
				</div>
				<div class="font-normal">
					total: <span class="text-sm font-bold">{total}</span>
				</div>
			</Tt.Content>
		</Tt.Root>
	</a>
{/snippet}

<div class="hidden w-full flex-col items-end justify-end md:flex md:h-[91px]">
	{#if totalScores > 0}
		<div
			class="mt-1 ml-3 text-sm -tracking-wider text-accent-foreground/55 italic"
		>
			mentioned most {preposition}:
		</div>

		<div class="items-row flex flex-row">
			<div class="mt-2 ml-3 space-x-2">
				<div class="flex flex-row -space-x-1">
					{#each scores?.slice(0, PREVIEWS_COUNT) as subentry}
						<div>
							{@render Hoverable({
								login: subentry.login,
								name: subentry.name,
								color: subentry.color,
								imgUrl: subentry.image,
								total: subentry.score,
							})}
						</div>
					{/each}
				</div>
				{#if totalScores > PREVIEWS_COUNT}
					{#if variant === "Channel"}
						<a href={`${Rh.getTenantedURL(login, host)}`}>
							<span
								class="ml-1 text-sm font-semibold -tracking-wider text-accent-foreground/55
                            italic transition-all duration-100 ease-in hover:text-accent-foreground/90"
							>
								{getMentionedSubtitle(totalScores)}
							</span>
						</a>
					{:else}
						<div>
							<span
								class="ml-1 text-sm font-semibold -tracking-wider text-accent-foreground/55
                                italic transition-all duration-100 ease-in hover:text-accent-foreground/90
                                cursor-default"
							>
								{getMentionedSubtitle(totalScores)}
							</span>
						</div>
					{/if}
				{:else}
					<div class="invisible">
						<span
							class="ml-1 text-sm font-semibold -tracking-wider text-accent-foreground/55
                            italic transition-all duration-100 ease-in hover:text-accent-foreground/95"
						>
						</span>
					</div>
				{/if}
			</div>
		</div>
	{:else}
		<div
			class="mt-1 ml-3 flex h-full items-center justify-center text-sm -tracking-wider
            text-accent-foreground/55 italic"
		>
			<div>never mentioned...</div>
		</div>
	{/if}
</div>
