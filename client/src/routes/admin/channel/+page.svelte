<script lang="ts">
	import * as InputGroup from "$lib/shadcn-components/ui/input-group/index";
	import Button from "$lib/shadcn-components/ui/button/button.svelte";
	import {
		CloudAlertIcon,
		CloudBackupIcon,
		CloudSyncIcon,
		PlusIcon,
	} from "@lucide/svelte";
	import Label from "$lib/shadcn-components/ui/label/label.svelte";
	import Loading from "$lib/components/loading/loading.svelte";
	import { enhance } from "$app/forms";
	import Panel from "$lib/components/admin/settings/panel.svelte";
	import ChannelSettings from "$lib/components/admin/settings/channel-settings.svelte";

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

	let { data } = $props();

	let waiting = $state(false);
	let configs = $derived.by(() => {
		const sortedConfigs = data.configs.sort((a, b) =>
			a.login.localeCompare(b.login)
		);

		const merged = sortedConfigs.map((cfg) => {
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
				hook: hookInfo.length > 0 ? hookInfo : null,
			};
		});

		return merged;
	});

	let filter = $state("");
	let cfgs = $derived.by(() => {
		const cfgs = configs;
		if (filter !== "") {
			return cfgs.filter((el) => el.login.includes(filter));
		}

		return cfgs;
	});

	function setWaiting(state?: boolean) {
		if (state != null) {
			waiting = state;
		} else {
			waiting = !waiting;
		}
	}
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

<div
	class="mt-8 flex w-full flex-col justify-start space-y-2 space-x-6 px-18 md:flex-row md:space-y-0"
>
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
			disabled={waiting}
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
	<form
		class="flex flex-row items-center space-x-2"
		action="?/deletehooks"
		method="POST"
		use:enhance={() => {
			waiting = true;

			return async ({ update }) => {
				await update();
				waiting = false;
			};
		}}
	>
		<Label for="clear-hooks">clear hooks</Label>
		<Button
			id="clear-hooks"
			type="submit"
			class="rounded-full"
			size="icon-sm"
			variant="outline"
			disabled={waiting}
		>
			<CloudAlertIcon class="size-5 text-red-800" />
		</Button>
	</form>
</div>
<div class="mb-16">
	<div class="my-4 w-full border-b-2"></div>
	{#each cfgs as cfg, idx (cfg.id)}
		<Panel config={cfg} {idx}>
			<ChannelSettings config={cfg} {waiting} {setWaiting} />
		</Panel>
	{/each}
</div>
