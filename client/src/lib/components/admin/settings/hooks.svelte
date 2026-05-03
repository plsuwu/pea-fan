<script lang="ts">
	import type { HookInfo } from "./types";
	import dayjs from "dayjs";
	import { cn } from "tailwind-variants";

	const INDICES = new Array(2);
	const FORMAT_STR = "HH:mm A, YY-MM-DD";
	const dateFmt = (date: string) => dayjs(date).format(FORMAT_STR);

	let { hook }: { hook?: HookInfo[] } = $props();

	function getHookEntry(index: number, hooks?: HookInfo[]) {
		if (hooks == null || index > hooks.length - 1) {
			return undefined;
		} else {
			return hooks[index];
		}
	}
</script>

{#snippet HookInfo(h: HookInfo)}
	<div class="flex w-full flex-row items-center">
		<div class="w-full self-start">
			{dateFmt(h.created_at)}
		</div>
		<div class="w-full self-start">
			{h?.type ?? "hook error"}
		</div>
		<div
			class={cn(
				"w-full self-end",
				h != null ? "text-foreground" : "text-accent"
			)}
		>
			[{h?.status ?? ""}]
		</div>
	</div>
{/snippet}

<div class="flex w-full flex-col self-start justify-self-start py-4">
	<div class="flex flex-col space-y-1 text-start text-xs">
		{#each INDICES as i}
			{@const h = getHookEntry(i, hook)}
			{#if h}
				{@render HookInfo(h)}
			{:else}
				<div
					class="flex h-full flex-row items-center self-center
                    text-red-600/75 italic"
				>
					missing hook info
				</div>
			{/if}
		{/each}
	</div>
</div>
