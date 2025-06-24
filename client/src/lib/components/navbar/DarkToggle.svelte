<script lang="ts">
	import { toggleMode, mode } from 'mode-watcher';
	import { Sun, CloudMoon } from 'phosphor-svelte';
	import Key from '$lib/components/Key.svelte';
</script>

<button
	onclick={toggleMode}
	class="duration-100 ease-out hover:brightness-90 active:scale-[0.98] active:transition-all"
>
	<Key class="h-8 w-8">
		{#if mode.current === 'dark'}
			<Sun size={16} class="dark:text-background/55 text-foreground/55" />
		{:else}
			<CloudMoon
				size={14}
				class="dark:text-background/55 text-foreground/55"
			/>
		{/if}
	</Key>
</button>
