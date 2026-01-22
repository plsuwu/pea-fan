<script lang="ts">
	import { onMount } from "svelte";
	import { clientLogger } from "$lib/observability/client/logger";
	import type { PageProps } from "./$types";

	let { data, error } = $props();

	onMount(() => {
		clientLogger.error("CLIENT_ERRORED", { data: error });
	});
</script>
