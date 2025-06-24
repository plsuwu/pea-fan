<script lang="ts">
	import { Button } from 'bits-ui';
	import { MagnifyingGlass } from 'phosphor-svelte';
	import { fade } from 'svelte/transition';
	import { expoIn, expoInOut } from 'svelte/easing';
	import SearchModal from './SearchModal.svelte';

	let open = $state(false);

	function toggleModal() {
		open = !open;
	}

	function handleKeyDown(event: KeyboardEvent) {
		if (event.ctrlKey && event.shiftKey && event.code === 'KeyK' && !open) {
			event.preventDefault();
			toggleModal();
		}

		if (event.code === 'Escape' && open) {
			event.preventDefault();
			toggleModal();
		}
	}
</script>

<svelte:window onkeydown={handleKeyDown} />

<Button.Root
	onclick={toggleModal}
	class="group border-border rounded-input hover:border-foreground-alt/40 flex min-w-[255px] flex-row items-center justify-start border px-4 py-2 duration-100 ease-in-out hover:brightness-90 active:scale-[0.98] active:transition-all"
>
	<MagnifyingGlass
		weight="bold"
		class="text-foreground-alt/50 group-hover:text-foreground-alt mr-4 transition-all duration-100 ease-out"
	/>
	<div
		class="text-foreground-alt/50 group-hover:text-foreground-alt text-sm transition-all"
	>
		search
	</div>
</Button.Root>

{#if open}
	<div
		role="button"
		onclick={toggleModal}
		onkeydown={handleKeyDown}
		tabindex="0"
		in:fade={{ delay: 0, duration: 150, easing: expoInOut }}
		out:fade={{ delay: 0, duration: 150, easing: expoIn }}
		class="absolute top-0 left-0 z-[99] flex h-full w-full flex-col items-center justify-center overflow-hidden bg-black/30 backdrop-blur-[2px]"
	></div>
	<div class="absolute z-[100] self-center top-[50%] left-[50%] -translate-[50%] content-center">
		<SearchModal />
	</div>
{/if}
