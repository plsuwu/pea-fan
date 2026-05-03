<script lang="ts">
	import Button from "$lib/shadcn-components/ui/button/button.svelte";
	import Checkbox from "$lib/shadcn-components/ui/checkbox/checkbox.svelte";
	import { CloudSyncIcon, FishingHookIcon } from "@lucide/svelte";
	// import { Snippet } from "svelte";
	import ActionHandler from "./action-handler.svelte";
	import Hooks from "./hooks.svelte";

	let { config, waiting, setWaiting } = $props();
</script>

<!-- {#snippet Action( -->
<!-- 	route: string, -->
<!-- 	label: string, -->
<!-- 	name: string, -->
<!-- 	children: Snippet -->
<!-- )} -->
<!---->
<!-- {/snippet} -->

<div class="flex w-[220px] flex-col justify-items-center space-y-2.5">
	<ActionHandler
		{waiting}
		{setWaiting}
		value={config.id}
		enabled={!waiting}
		name="channel-id"
		action="?/bot"
		label="replying"
	>
		<input type="hidden" name="channel-id" value={config.id} />
		<Checkbox
			id={config.id}
			value={config.id}
			checked={config.enabled}
			class="size-[19px]"
			type="submit"
			formaction="?/bot"
			disabled={waiting}
		/>
	</ActionHandler>

	<ActionHandler
		{waiting}
		{setWaiting}
		value={config.id}
		enabled={!waiting}
		name="channel-id"
		action="?/sync"
		label="resync state"
	>
		<div class="flex flex-row items-center">
			<button
				value={config.id}
				class="items-center transition-all duration-150 hover:brightness-50"
				name="channel-id"
				type="submit"
				disabled={waiting}
			>
				<CloudSyncIcon size={20} />
			</button>
		</div>
	</ActionHandler>

	<ActionHandler
		{waiting}
		{setWaiting}
		value={config.id}
		enabled={!waiting}
		name="channel-id"
		action="?/sync"
		label="reset hook"
	>
		<button
			value={config.id}
			class="items-center transition-all duration-150 hover:brightness-50"
			name="channel-id"
			type="submit"
			disabled={waiting}
		>
			<FishingHookIcon size={20} />
		</button>
	</ActionHandler>
</div>
