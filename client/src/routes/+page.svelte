<script lang="ts">
	import { onMount } from "svelte";
	import { clientLogger as log } from "$lib/observability/client/logger";
    
    onMount(() => {
        log.trace("hiii");
        log.debug("hiii");
        log.info("hiii");
        log.warn("hiii");
        log.error("hiii");
        log.fatal("hiii");
    });
</script>

<div>hello :))</div>
