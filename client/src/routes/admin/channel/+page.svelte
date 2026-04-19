<script lang="ts">
	import { enhance } from "$app/forms";
	import * as InputGroup from "$lib/shadcn-components/ui/input-group/index";
	import Button from "$lib/shadcn-components/ui/button/button.svelte";

	import { readableColor } from "$lib/utils";
	import { CloudBackupIcon, CloudSyncIcon, PlusIcon } from "@lucide/svelte";
	import Label from "$lib/shadcn-components/ui/label/label.svelte";
	import Checkbox from "$lib/shadcn-components/ui/checkbox/checkbox.svelte";
	import { fly } from "svelte/transition";
	import { expoOut } from "svelte/easing";
	import { cn } from "tailwind-variants";
	import { mode } from "mode-watcher";
	import dayjs from "dayjs";
	import Loading from "$lib/components/loading/loading.svelte";

	type HookInfo = {
		id: string;
		type: "stream.online" | "stream.offline";
		status: "enabled" | string;
		version: "1";
		cost: number | string;
		condition: {
			broadcaster_user_id: string;
		};
		created_at: string;
	};

	let { form, data } = $props();
	// let live = $derived();
	let waiting = $state(false);
	let configs = $derived.by(() => {
		const sortedConfigs = data.configs.sort((a, b) =>
			a.login.localeCompare(b.login)
		);

		// const sortedHooks = (data.hooks as [string, any][]).sort(
		//         ([ak, _a], [bk, _b]) => ak.localeCompare(bk)
		// );'

		const merged = sortedConfigs
			.map((cfg) => {
				const hookInfo = (data.hooks as [string, HookInfo][])
					.map((hook) => {
						if (hook[0] === cfg.login) {
							return hook[1];
						}
					})
					.filter(Boolean);

				const isLive =
					data.liveBroadcasters.find((br) => br.id === cfg.id) != null
						? true
						: false;

				return {
					...cfg,
					live: isLive,
					hook: hookInfo,
				};
			})
			.filter((cfg) => cfg.hook.length > 0);

		return merged;
	});

	let createResult: { success: boolean; result?: string } | undefined =
		$state(undefined);

	let filter = $state("");
	let cfgs = $derived.by(() => {
		const cfgs = configs;
		if (filter !== "") {
			return cfgs.filter((el) => el.login.includes(filter));
		}

		return cfgs;
	});
</script>

<Loading {waiting} />

<div class="flex flex-col space-x-4 px-8 md:flex-row md:items-center">
	<form
		action="?/create"
		method="POST"
		use:enhance={() => {
			waiting = true;

			return async ({ update }) => {
				await update();
				waiting = false;
			};
		}}
	>
		<InputGroup.Root class="w-[250px] rounded-3xl">
			<InputGroup.Input name="channel" type="text" placeholder="new channel" />
			<InputGroup.Addon align="inline-end">
				<InputGroup.Button
					type="submit"
					class="rounded-full"
					size="icon-sm"
					disabled={waiting}
				>
					<PlusIcon />
				</InputGroup.Button>
			</InputGroup.Addon>
		</InputGroup.Root>
	</form>

	<div>
		<Label for="filter" class="w-[250px]"></Label>
		<InputGroup.Root class="w-[250px] rounded-3xl">
			<InputGroup.Input
				bind:value={filter}
				id="filter"
				name="filter"
				type="text"
				placeholder="filter by login"
			/>
		</InputGroup.Root>
	</div>
</div>

<div class="mt-8 flex w-full flex-row justify-start space-x-6 px-18">
	<form
		class="flex flex-row items-center space-x-2"
		method="POST"
		action="?/sync"
		use:enhance={() => {
			waiting = true;

			return async ({ update }) => {
				await update();
				waiting = false;
			};
		}}
	>
		<Label for="sync-all">sync all</Label>
		<Button
			id="sync-all"
			type="submit"
			value="all"
			name="channel-id"
			variant="outline"
			size="icon-sm"
			class="rounded-full"
			aria-label="sync all"
		>
			<CloudSyncIcon class="size-5" />
		</Button>
	</form>
	<form
		class="flex flex-row items-center space-x-2"
		action="?/resethooks"
		method="POST"
		use:enhance={() => {
			waiting = true;

			return async ({ update }) => {
				await update();
				waiting = false;
			};
		}}
	>
		<Label for="reset-hooks">reset hooks</Label>
		<Button
			id="reset-hooks"
			type="submit"
			class="rounded-full"
			size="icon-sm"
			variant="outline"
			disabled={waiting}
		>
			<CloudBackupIcon class="size-5" />
		</Button>
	</form>
</div>
<div class="mb-16 px-12">
	<div class="my-4 w-full border-b-2"></div>
	{#each cfgs as cfg, idx (cfg.id)}
		<div
			class={cn(
				`grid grid-cols-3 items-center space-x-5 px-8 py-1.5 transition-all
            duration-300 ease-in-out md:grid-cols-6`,
				idx % 2 === 0 ? "bg-accent/25" : ""
			)}
		>
			<div
				class={cn(
					"flex w-max flex-row items-center self-center text-start font-bold transition-transform duration-300 ease-in-out",
					cfg.live ? "text-red-500" : "text-foreground"
				)}
			>
				{cfg.login}
			</div>

			<div
				class="hidden w-max text-start transition-transform duration-300 ease-in-out md:block"
			>
				{cfg.id}
			</div>

			<div class="mx-2 hidden w-full flex-row md:flex">
				<div class="ml-4 h-[20px] w-[20px] border-l-2"></div>
				<div
					class="mx-2 hidden w-full justify-start rounded-md px-4 font-bold
                    transition-transform duration-300 ease-in-out md:flex"
					style={`color: ${readableColor(cfg.color, mode.current === "dark" ? "light" : "dark", 12)};
                    background-color: ${readableColor(cfg.color, mode.current, 11)};`}
				>
					{cfg.color}
				</div>
			</div>

			<div
				class="col-span-2 flex w-full flex-row items-center justify-between justify-self-start"
			>
				<div class="ml-4 h-[20px] w-[20px] border-l-2"></div>
				<div
					class="flex w-full flex-col items-center justify-center rounded-md px-4 md:flex-row"
				>
					<div class="flex w-[350px] flex-col space-y-1 text-start text-xs">
						{#each Array(0, 1) as idx}
							<div class="flex w-full flex-row items-center">
								<div class="w-full self-start">
									{cfg.hook
										? dayjs(cfg.hook?.[idx]?.created_at).format(
												"HH:mm A, YY-MM-DD"
											)
										: ""}
								</div>
								<div class="w-full self-start">
									{cfg.hook?.[idx]?.type ?? ""}
								</div>
								<div
									class={cn(
										"w-full self-end",
										cfg.hook != null ? "text-foreground" : "text-accent"
									)}
								>
									[{cfg.hook?.[idx]?.status ?? "disabled"}]
								</div>
							</div>
						{/each}
					</div>
				</div>
			</div>

			<div class="col-span-1 flex w-full flex-row items-center">
				<div class="ml-4 h-[20px] w-[20px] border-l-2"></div>
				<form
					method="POST"
					action="?/bot"
					class="flex flex-col items-center space-y-2"
					use:enhance={() => {
						waiting = true;

						return async ({ update }) => {
							await update();
							waiting = false;
						};
					}}
				>
					<div class="flex w-full flex-row">
						<Label for="bot-id" class="mr-4 pl-8 text-xs md:text-sm"
							>replies</Label
						>
						<div class="flex flex-row items-center">
							<input type="hidden" name="channel-id" value={cfg.id} />
							<Checkbox
								value={cfg.id}
								checked={cfg.enabled}
								type="submit"
								formaction="?/bot"
							/>
						</div>
					</div>
				</form>
				<div class="ml-4 h-[20px] w-[20px] border-l-2"></div>
				<form
					method="POST"
					action="?/sync"
					class="flex flex-col items-center space-y-2"
					use:enhance={() => {
						waiting = true;
						return async ({ update }) => {
							await update({ reset: false });
							waiting = false;
						};
					}}
				>
					<div
						class="flex w-full flex-row justify-end justify-self-end
                                md:justify-between"
					>
						<Label for="sync" class="mr-4 pl-8 text-xs md:text-sm">sync</Label>
						<div class="flex flex-row items-center">
							<button
								value={cfg.id}
								name="channel-id"
								type="submit"
								class="items-center transition-all duration-150 hover:brightness-50"
							>
								<CloudSyncIcon size={18} />
							</button>
						</div>
					</div>
				</form>
			</div>
		</div>
	{/each}
</div>
