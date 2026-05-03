<script lang="ts">
	let { config } = $props();
</script>

<div class="text-lg font-medium" class:live-broadcaster={config.live}>
	{config.login}
</div>

<style>
	.live-broadcaster {
		color: var(--color-red-500);
	}
</style>
