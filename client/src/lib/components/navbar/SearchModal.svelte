<script lang="ts">
	import { Command } from 'bits-ui';

	let { channels, chatters } = $props();

	let value = $state('');
</script>

<Command.Root
	class="divide-border border-muted bg-background flex h-[450px] w-[650px] flex-col divide-y self-start 
    overflow-hidden rounded-xl border"
>
	<Command.Input
		autofocus
		class="focus-override h-input placeholder:text-foreground-alt/50 bg-background focus:outline-hidden inline-flex
        shrink-0 truncate rounded-tl-xl rounded-tr-xl px-4 text-sm transition-colors focus:ring-0"
	/>
	<Command.List class="overflow-y-auto overflow-x-hidden px-2 pb-2">
		<Command.Viewport>
			<Command.Empty
				class="text-muted-foreground flex w-full items-center justify-center pb-6 pt-8 text-sm"
				>://</Command.Empty
			>

			<Command.Group>
				<Command.GroupHeading
					class="text-muted-foreground px-3 pb-2 pt-4 text-xs"
				>
					Channels
				</Command.GroupHeading>
				<Command.GroupItems>
					{#each channels as ch}
						<Command.Item
							class="rounded-button data-selected:bg-muted outline-hidden flex h-10 cursor-pointer
                            select-none items-center gap-1 px-3 py-0.5 text-sm"
						>
							<img
								src={ch.image}
								class="size-7 rounded-full"
								alt={`${ch.broadcaster} profile`}
							/>
							{ch.broadcaster}
						</Command.Item>
					{/each}
				</Command.GroupItems>
			</Command.Group>
			<Command.Group>
				<Command.GroupHeading
					class="text-muted-foreground px-3 pb-2 pt-4 text-xs"
				>
					Chatters
				</Command.GroupHeading>
				<Command.GroupItems>
					{#each chatters as ch}
						<Command.Item
							class="rounded-button data-selected:bg-muted outline-hidden flex h-10 cursor-pointer
                            select-none items-center gap-1 px-3 py-0.5 text-sm"
						>
							<img
								src={ch.image}
								class="size-7 rounded-full"
								alt={`${ch.login} profile`}
							/>
							{ch.login}
						</Command.Item>
					{/each}
				</Command.GroupItems>
			</Command.Group>
		</Command.Viewport>
	</Command.List>
</Command.Root>
