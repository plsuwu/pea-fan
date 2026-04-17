<script lang="ts">
	import type { SystemModeValue } from "mode-watcher";
	import { type UntypedEntry, readableColor } from "$lib/utils";
	import { page } from "$app/state";
	import { cn } from "$lib/shadcn-components/utils";
	import { Rh } from "$lib/utils/route";
	import Ranking from "./ranking.svelte";
	import ProfileImage from "./profile-image.svelte";
	import SubTable from "./subtable.svelte";
	import Total from "./total.svelte";
	import Name from "./name.svelte";
	import Link from "./link.svelte";

	let {
		entry,
		variant,
		mode,
		showScoreIcons = true,
	}: {
		entry: UntypedEntry & { isLive: boolean };
		variant: "Channel" | "Chatter";
		mode: SystemModeValue;
		showScoreIcons?: boolean;
	} = $props();

	let isChannel = $derived(variant === "Channel");
	let href = $derived(`${Rh.getTenantedURL(entry.login, page.url.host)}`);
	let nameElement: HTMLDivElement;
	let ringedElement: HTMLImageElement | undefined = $state();

	let entryColor = $derived(readableColor(entry.color));

	function handleMouseEvent(event: MouseEvent) {
		if (event.type === "mouseover") {
			nameElement.setAttribute(
				"style",
				`color: ${entryColor}; 
                transition-property: all; 
                transition-duration: 300ms;
                transition-timing-function: var(--tw-ease, var(--default-transition-timing-function));`
			);
			if (ringedElement) {
				ringedElement.setAttribute(
					"style",
					`--tw-ring-color: ${entryColor};
                    transition-property: all; 
                    transition-duration: 300ms;
                    transition-timing-function: var(--tw-ease, var(--default-transition-timing-function));`
				);
			}
		} else {
			nameElement.setAttribute(
				"style",
				`color: var(--foreground);
                transition-property: all; 
                transition-duration: 300ms;
                transition-timing-function: var(--tw-ease, var(--default-transition-timing-function));`
			);
			if (ringedElement) {
				ringedElement.setAttribute(
					"style",
					`--tw-ring-color: var(--foreground);
                    transition-property: all; 
                    transition-timing-function: var(--tw-ease, var(--default-transition-timing-function));
                    transition-duration: 300ms;
                    `
				);
			}
		}
	}

	let subtableScores = $derived.by(() => {
		return entry.scores.map((entry) => {
			return {
				...entry,
				color: readableColor(entry.color, mode),
			};
		});
	});

	const containerClass = $derived(
		mode === "light" ? "shadow-container" : "shadow-container-dark"
	);
</script>

<div
    id={entry.ranking.toString()}
	class={cn(
		`relative flex items-center justify-between space-x-4 overflow-x-hidden
        rounded-[1px] border-2 border-foreground bg-background py-4 pl-4 transition-all
        duration-300 ease-out hover:-translate-y-4 md:pl-6`,
		containerClass
	)}
>
	<div class="group flex h-full w-max min-w-0 grow flex-row justify-between">
		<Link href={isChannel ? href : undefined} {handleMouseEvent}>
			<div class="mr-6">
				<Ranking ranking={entry.ranking} isLive={entry.isLive} />
			</div>
			<div class="flex-none transition-all duration-300 ease-in-out">
				<ProfileImage
					image={entry.image}
					alt={entry.login}
					bind:element={ringedElement}
				/>
			</div>
			<div
				class="w-max flex-1 transition-all duration-300 ease-in-out"
				bind:this={nameElement}
			>
				<Name isLive={entry.isLive} name={entry.name} />
					
			</div>
		</Link>
	</div>
	<div
		class="z-10 flex flex-1 shrink-0 grow flex-row items-center bg-linear-to-r from-background/5
        via-background/99 to-background pr-4 pl-8 text-right md:pr-6"
	>
		{#if showScoreIcons}
			<div class="hidden w-full text-nowrap md:flex">
				<SubTable
					{variant}
					login={entry.login}
					scores={subtableScores}
					totalScores={entry.totalScores}
				/>
			</div>
		{/if}
		<div class="flex-1 md:min-w-[125px]">
			<Total total={entry.total} />
		</div>
	</div>
</div>
