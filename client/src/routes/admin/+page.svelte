<script lang="ts">
	import { enhance } from "$app/forms";
	import * as InputGroup from "$lib/shadcn-components/ui/input-group/index";
	import { readableColor } from "$lib/utils";
	import { SearchIcon } from "@lucide/svelte";

	let { form, data } = $props();
	let waiting = $state(false);

	let helixResult: any | undefined = $state(undefined);
	let databaseResult: any[] | undefined = $state(undefined);
</script>

{#snippet SearchResult(title: string, content: string)}
	<div class="flex flex-row justify-between">
		<div>{title}</div>
		<div>{content}</div>
	</div>
{/snippet}

<div class="mx-10 mt-12"></div>
<div class="flex flex-col">
	<form
		action="?/helix"
		method="POST"
		use:enhance={() => {
			waiting = true;

			return async ({ update }) => {
				await update();
				if (form && form.data) {
					helixResult = form.data;
				}

				waiting = false;
			};
		}}
	>
		<InputGroup.Root class="w-[300px] rounded-full">
			<InputGroup.Input
				type="text"
				autocomplete="off"
				name="user"
				placeholder="helix user search (id or login)"
			/>

			<InputGroup.Addon align="inline-end">
				<InputGroup.Button
					class="rounded-full"
					size="icon-sm"
					type="submit"
					disabled={waiting}
				>
					<SearchIcon />
				</InputGroup.Button>
			</InputGroup.Addon>
		</InputGroup.Root>
	</form>
	<div class="mt-8 p-6"></div>
	<div
		class="h-[200px] w-[400px] overflow-y-scroll rounded-md border
    border-accent px-8"
	>
		{#if helixResult}
			{#if helixResult.length === 0}
				no results.
			{:else}
				{#each helixResult as result}
					<div class="flex flex-col space-y-1 py-3">
						{@render SearchResult("display_name", result.display_name)}
						{@render SearchResult("login", result.login)}
						{@render SearchResult("id", result.id)}
						<div class="flex flex-row justify-between">
							<div>color</div>
							<div style:color={`${readableColor(result.color)}`}>
								{result.color}
							</div>
						</div>
						<div class="flex flex-row items-center justify-between">
							<div>profile_image</div>
							<div>
								<img
									src={result.profile_image_url}
									alt={`${result.login}`}
									class="size-12 rounded-full"
								/>
							</div>
						</div>
					</div>
				{/each}
			{/if}
		{/if}
	</div>
</div>

<div class="mx-10 mt-12"></div>
<div class="flex flex-col">
	<form
		action="?/database"
		method="POST"
		use:enhance={() => {
			waiting = true;

			return async ({ update }) => {
				await update();
				if (form && form.data) {
					databaseResult = form.data;
				}

				console.log(databaseResult);
				waiting = false;
			};
		}}
	>
		<InputGroup.Root class="w-[300px] rounded-full">
			<InputGroup.Input
				type="text"
				autocomplete="off"
				name="user"
				placeholder="helix user search (id or login)"
			/>

			<InputGroup.Addon align="inline-end">
				<InputGroup.Button
					class="rounded-full"
					size="icon-sm"
					type="submit"
					disabled={waiting}
				>
					<SearchIcon />
				</InputGroup.Button>
			</InputGroup.Addon>
		</InputGroup.Root>
	</form>
	<div class="mt-8 p-6"></div>
	<div
		class="h-[400px] w-[400px] overflow-y-scroll rounded-md border
    border-accent px-8"
	>
		{#if databaseResult}
			{#if databaseResult.length === 0}
				no results.
			{:else}
				{#each databaseResult as result}
					<div class="mb-3 flex flex-col space-y-1 pt-4">
						{@render SearchResult("name", result.name)}
						{@render SearchResult("login", result.login)}
						{@render SearchResult("id", result.id)}
						{@render SearchResult("total", result.total)}
						{@render SearchResult("ranking", result.ranking)}
						<div class="flex flex-row justify-between">
							<div>color</div>
							<div style:color={`${readableColor(result.color)}`}>
								{result.color}
							</div>
						</div>
						<div class="flex flex-row items-center justify-between">
							<div>image</div>
							<div>
								<img
									src={result.image}
									alt={`${result.login}`}
									class="size-12 rounded-full"
								/>
							</div>
						</div>
						<div class="border-b pb-4"></div>
					</div>
				{/each}
			{/if}
		{/if}
	</div>
</div>
