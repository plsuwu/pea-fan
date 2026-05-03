<script lang="ts">
	import { readableColor } from "$lib/utils";
	import { mode } from "mode-watcher";

	let { config } = $props();
</script>

<div
	class="mt-2 flex w-full flex-row justify-between rounded-lg px-4 py-px font-bold"
	style={`
        color: ${readableColor(config.color, mode.current === "dark" ? "light" : "dark", 12)};
        background-color: ${readableColor(config.color, mode.current, 11)};
    `}
>
	<div>
		{config.id}
	</div>
	<div>
		{config.color}
	</div>
</div>
