<script>
	import cacheImg from "$lib/assets/img/funny-cache.webp";
	import { slide } from "svelte/transition";
	import { Input } from "$lib/shadcn-components/ui/input";
	import Button from "$lib/shadcn-components/ui/button/button.svelte";
	import { ChevronUp } from "@lucide/svelte";
	import { expoIn, expoInOut, expoOut } from "svelte/easing";
	import Kbd from "$lib/components/misc/kbd.svelte";
	import { enhance } from "$app/forms";

	let detailScoresOpen = $state(false);
</script>

<div class="mb-10">
	<div class="flex w-full flex-col items-center justify-center px-6">
		<div class="my-12">
			<span class="text-base font-semibold xl:text-xl">in short:</span> i had a really
			bad system in place such that i kept losing track of chatters (heh) - please
			fill out the form and i will look into adjusting your score (i would also like
			to apologize for across-the-board score reductions but you live and you learn
			and so on).
		</div>
		<form
			method="POST"
			class="flex w-full flex-col space-y-1 self-center sm:w-3/5 xl:w-1/2"
            use:enhance={({ }) => {
            
                }
            }
		>
			<Input
				name="current"
				type="text"
				placeholder="current twitch username"
				class="placeholder:text-sm lg:placeholder:text-base"
				required
			/>
			<Input
				type="text"
				name="previous"
				placeholder="previous twitch username(s)"
				class="placeholder:text-sm lg:placeholder:text-base"
			/>
			<Input
				type="number"
				name="score"
				placeholder="expected score"
				required
				class="placeholder:text-sm lg:placeholder:text-base"
			/>

			<Input
				type="text"
				name="comment"
				placeholder="additional comments"
				class="placeholder:text-sm lg:placeholder:text-base"
			/>
			<Button
				variant="outline"
				size="sm"
				class="mt-1 w-max self-end rounded-full"
				type="submit"
			>
				send
			</Button>
		</form>
		<div class="my-10 space-y-2 text-justify text-muted-foreground">
			<p>
				note that i will be manually processing these requests!! (what could go
				wrong)
			</p>
		</div>
	</div>
</div>
<button
	onclick={() => (detailScoresOpen = !detailScoresOpen)}
	class="mb-4 flex w-full flex-row items-center justify-center space-x-4 self-center
    text-start text-lg font-semibold transition-all duration-200 ease-in-out
    hover:cursor-pointer hover:opacity-50"
>
	<ChevronUp
		class={`shrink-0 transition-all duration-500 ease-out ${
			detailScoresOpen ? "rotate-180" : ""
		}`}
	/>
	<div>
		{detailScoresOpen ? "less" : "more"} info about this (blogpost-esque rambling)
	</div>
</button>
<div class="mt-8 w-full border-b"></div>
{#if detailScoresOpen}
	<div
		in:slide={{ duration: 500, axis: "y", easing: expoInOut }}
		out:slide={{ duration: 550, axis: "y", easing: expoInOut }}
		class="rounded-b-lg border-r border-b border-l p-4 text-foreground/80"
	>
		<p>
			As God's most impatient creation, I did a pretty hacky job on the last
			version of this website by deciding to use each account's
			<Kbd>display_name</Kbd> to track users and their pisscounts in a Redis cache.
			For context: your account's <Kbd>display_name</Kbd> is the version of your Twitch
			name that handles capitalization and localization, which means I more or less
			lost your "progress" (so to speak) not only if you were to change your name
			on Twitch - but even if you simply changed, like, it's capitalization:
		</p>

		<img
			src={cacheImg}
			alt="score storage using display_name example"
			class="mx-auto my-10 border-2 object-fill p-3"
		/>

		<p>
			This was also the case for channel users, which has been (frankly) the
			worst thing I have had to deal with in possibly my entire life.
		</p>

		<p class="mt-4">
			I've worked <i>vewy hawd</i> to transition this over to a far more robust
			PostgreSQL database (which has also solved a handful of other issues that
			I don't think I will get into here) - but we are now most importantly
			using your Twitch account's
			<Kbd>id</Kbd> (an immutable identifier!!) instead. Tragically, though, I need
			to pull account data from Twitch's API in order to transition scores over, which
			is more or less completely impossible to do when an account's name change involved
			anything more than just altering the letter casing.
		</p>

		<p class="mt-4">
			I've tried to go through and manually fix up any scores for people I know,
			but I am not omniscient!!
		</p>

		<p class="mt-4">
			Alternatively, if there is a method to retrieve an account's current <Kbd
				>id</Kbd
			> from a historic name that I am unaware of, please also let me know...
		</p>
	</div>
{/if}

<style>
	img {
		border-radius: var(--radius-md);
	}
</style>
