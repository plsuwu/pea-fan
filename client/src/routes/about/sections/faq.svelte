<script lang="ts">
	import { slide } from "svelte/transition";
	import { ChevronUp } from "@lucide/svelte";
	import { expoInOut } from "svelte/easing";

	let current: undefined | number = $state(undefined);

	const FAQ_ITEMS = [
		{
			q: "why",
			a: "my twitch username looks like it says piss",
		},
		{
			q: "does it only count while a broadcaster is live?",
			a: "i think so",
		},
		{
			q: "what are you actually counting?",
			a: `each message sent in any of the tracked channels containing the word
            "piss" is one point - this is added to the channel's total and your
            personal chatter total:
                <ul class="list-disc list-inside my-8 px-4">
                    <li><b>"piss"</b> &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp; --> yes!</li>
                    <li><b>"PISS"</b> &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp; --> yes!</li>
                    <li><b>"foopissbar"</b>&nbsp; --> yes!</li>
                    <li><b>"pi ss"</b> &nbsp;&nbsp;&nbsp;&nbsp;&nbsp;  --> no..</li>
                </ul>
            there is a maximum of one point per chat message, so mentioning piss multiple times
            in the same message will still be +1.
            `,
		},
	];
</script>

{#snippet CustomAccordion(idx: number, q: string, a: string)}
	<button
		onclick={() => (current = current === idx ? undefined : idx)}
		class="flex w-full flex-row items-center justify-start space-x-2 self-center
        border-b-2 pb-3 text-left text-lg font-semibold transition-opacity duration-200
        ease-out hover:cursor-pointer hover:opacity-50 lg:space-x-12 lg:text-3xl"
	>
		<ChevronUp
			class={`shrink-0 transition-all duration-550 ease-in-out ${
				current === idx ? "rotate-180" : ""
			}`}
		/>
		<div class="leading-tight">
			{q}
		</div>
	</button>
	{#if current === idx}
		<div
			in:slide={{ duration: 500, axis: "y", easing: expoInOut }}
			out:slide={{ duration: 500, axis: "y", easing: expoInOut }}
			class="mx-2 mb-10 flex min-h-[55px] flex-col justify-center rounded-b-lg border-r border-b
            border-l px-4 py-4 text-base text-accent-foreground/85 lg:mx-6 lg:min-h-[125px] lg:px-8
            lg:py-8 lg:text-xl"
		>
			{@html a}
		</div>
	{/if}
	<div class="mb-4"></div>
{/snippet}

<div class="mb-18">
	{#each FAQ_ITEMS as item, idx}
		{@render CustomAccordion(idx, item.q, item.a)}
	{/each}
</div>
