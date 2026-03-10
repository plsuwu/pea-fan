<script lang="ts">
	import * as BGrp from "$lib/shadcn-components/ui/button-group";
	import Button from "$lib/shadcn-components/ui/button/button.svelte";
	import { setModeCookie } from "$lib/utils/mode-cookie";
	import { Moon, Sun } from "@lucide/svelte";
	import { mode, setMode } from "mode-watcher";

	function toggleMode() {
		mode.current === "dark" ? setMode("light") : setMode("dark");
		setModeCookie(mode.current ?? "system");
	}
</script>

<BGrp.Root>
	<Button variant="ghost" onclick={toggleMode}>
		{#if mode.current === "dark"}
			<Moon />
		{:else}
			<Sun />
		{/if}
	</Button>
</BGrp.Root>
