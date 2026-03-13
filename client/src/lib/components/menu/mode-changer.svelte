<script lang="ts">
	import * as BGrp from "$lib/shadcn-components/ui/button-group";
	import Button from "$lib/shadcn-components/ui/button/button.svelte";
	import { setModeCookie } from "$lib/utils/mode-cookie.svelte";
	import { Moon, MoonIcon, Sun, SunIcon } from "@lucide/svelte";
	import { mode, toggleMode } from "mode-watcher";
</script>

{#if mode.current === "dark"}
	<SunIcon />
{:else}
	<MoonIcon />
{/if}
