<script lang="ts">
	import { Button } from 'bits-ui';
	import Key from '$lib/components/Key.svelte';
	import { MagnifyingGlass, Command, ArrowFatUp } from 'phosphor-svelte';
	import { fade } from 'svelte/transition';
	import { expoIn, expoInOut } from 'svelte/easing';
	import SearchModal from './SearchModal.svelte';

	let { data } = $props();
	let channels = data.channels;
	let chatters = data.chatters;

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
	class="h-10 border-border rounded-input hover:border-foreground-alt/40 group flex min-w-[255px]
    flex-row items-center justify-center border px-4 py-2 duration-100 ease-in-out hover:brightness-90
    active:scale-[0.98] active:transition-all"
>
	<MagnifyingGlass
		weight="bold"
		class="text-foreground-alt/50 group-hover:text-foreground-alt mr-4 transition-all duration-100
        ease-out"
	/>
	<div
		class="text-foreground-alt/50 group-hover:text-foreground-alt flex w-full flex-row items-center
        justify-between text-sm transition-all"
	>
		<div>search</div>
		<div class="flex flex-row flex-nowrap justify-self-end">
			<Key>
				<Command size={12} class="shrink-0" />
			</Key>
			<Key>
				<ArrowFatUp size={12} class="shrink-0" />
			</Key>
			<Key
				><span class="mt-px font-mono text-[12px] font-medium">K</span
				></Key
			>
		</div>
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
		class="absolute left-0 top-0 flex h-screen w-full flex-col items-center justify-center
            overflow-hidden bg-black/30 backdrop-blur-[2px]"
	></div>
	<div class="translate-[50%] top-[50%] left-[25%] absolute z-[101] content-center">
		<SearchModal {channels} {chatters} />
	</div>
{/if}
