<script lang="ts">
    import { page } from '$app/state';
	import clientLogger from '$lib/observability/client/logger';

    let url = $state(page.params.variant);

    function otelTest() {
        clientLogger.debug("hello");
    }
</script>

<button onclick={otelTest}>
    test
</button>
<div>
    current: <span class="font-medium bg-amber-400">{url}</span>
</div>
