<script lang="ts">
	import { page } from "$app/state";
	import clientLogger from "$lib/observability/client/logger";
    
    let error = $derived(page.error);

	$effect(() => {
        if (error) {
		    clientLogger.error("CLIENT_ERROR", { error });
        }
	});
</script>
