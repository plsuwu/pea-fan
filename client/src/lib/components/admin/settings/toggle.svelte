<script lang="ts">
	import { SettingsIcon } from "@lucide/svelte";

	interface Props {
		panelState: boolean;
		togglePanelState: () => void;
	}

	let { togglePanelState }: Props = $props();
</script>

<button
	onclick={togglePanelState}
	class="row-end-1 transition-opacity duration-200 ease-out hover:cursor-pointer
    hover:opacity-50"
>
	<SettingsIcon size={20} />
</button>
