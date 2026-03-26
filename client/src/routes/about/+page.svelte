<script lang="ts">
	import unresolvedChatters from "$lib/assets/unresolved-chatters.json";
	import cacheImg from "$lib/assets/img/funny-cache.webp";
	import { slide } from "svelte/transition";
	import { Input } from "$lib/shadcn-components/ui/input";
	import Separator from "$lib/shadcn-components/ui/separator/separator.svelte";
	import Button from "$lib/shadcn-components/ui/button/button.svelte";

	let missing = $derived(
		Object.keys(unresolvedChatters).sort((a, b) => a.localeCompare(b))
	);

	let detailScoresOpen = $state(false);
</script>

<div class="flex w-full flex-col items-center justify-center">
	<div class="w-full px-4 xl:w-1/2 xl:px-1">
		<h1 class="mb-4 text-4xl font-bold underline">incorrect scores</h1>
		<div class="mb-10">
			<h2 class="text-2xl font-semibold">
				hmu using the form below if ur score looks wrong and u care about it
			</h2>
			<div class="flex w-full flex-col items-center justify-center px-6">
				<div class="my-10 space-y-2 text-justify text-muted-foreground">
					<p>note that i will be manually processing any requests!!!</p>
					<p>
						in any case, this is kind of trust-based because i cannot be
						bothered setting up a twitch oauth flow BUT if it gets all fucked up
						i will be forced to set up oauth and you WILL have to log in before
						submitting a request and it will be ruined for everyone.
					</p>
				</div>
				<form
					method="POST"
					class="flex w-full flex-col space-y-1 self-center xl:w-1/2"
				>
					<Input
						name="current"
						type="text"
						placeholder="current twitch username"
						required
					/>
					<Input
						type="text"
						name="previous"
						placeholder="previous twitch username(s) (if applicable)"
					/>
					<Input type="text" name="score" placeholder="expected score" />
					<Button
						variant="outline"
						size="sm"
						class="mt-1 w-max self-end rounded-full"
						type="submit"
					>
						send
					</Button>
				</form>
			</div>
		</div>
		<button
			onclick={() => (detailScoresOpen = !detailScoresOpen)}
			class="mb-4 text-lg font-semibold transition-opacity duration-200 ease-out
            hover:cursor-pointer hover:opacity-50"
		>
			{detailScoresOpen ? "less" : "more"} info about this (similar to a blog post)
		</button>
		{#if detailScoresOpen}
			<div
				in:slide={{ duration: 250, axis: "y" }}
				out:slide={{ duration: 250, axis: "y" }}
			>
				<p>
					As God's most impatient creation, I did a pretty hacky job on the last
					version of this website and used an account's
					<kbd class="rounded-full bg-muted px-1.5 py-0.5">display_name</kbd>
					(i.e. the variant that Twitch uses for displaying capitalization and localization)
					to store pisscounts in a Redis cache. This means I more or less lost your
					"progress" (so to speak) not only if you were to change your name on Twitch
					- but even if you simply changed, like, it's capitalization, for example:
				</p>

				<img
					src={cacheImg}
					alt="score storage using display_name"
					class="mx-auto my-6 object-cover"
				/>

				<p>
					I've transitioned this over to a more robust (though significantly
					more complex) PostgreSQL database where users are defined around their
					<kbd class="rounded-full bg-muted px-1.5 py-0.5">id</kbd>. In order to
					do this, though, I need to pull account data from Twitch's API, which
					is more or less impossible to do when a name change was more than just
					altering the letter casing.
				</p>

				<p class="mt-8">
					I've tried to go through and manually fix up any scores for people I
					know, but I am not omniscient!!
				</p>
			</div>
		{/if}
	</div>
</div>

<style>
	img {
		border-radius: var(--radius-md);
	}
</style>
