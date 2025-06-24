<script lang="ts">
	import { Command } from 'bits-ui';

	let value = $state('');
	const channels = [
		'sleepiebug',
		'parasi',
		'unipiu',
		'cchiko_',
		'liljuju',
		'vacu0usly',
		'bexvalentine',
		'rena_chuu',
		'snoozy',
		'womfyy',
		'kyoharuvt',
		'batatvideogames',
		'myrmidonvt',
		'kokopimento',
		'myramors',
		'sheriff_baiken',
		'lcolonq',
		'chocojax',
		'miffygeist',
		'haelpc',
		'gloomybyte',
		'niupao',
		'souly_ch',
		'kyundere',
		'miaelou',
		'saltae',
		'flippersphd',
		'misspeggyx'
	];
</script>

<Command.Root
	class="divide-border border-muted bg-background flex h-[450px] w-[650px] flex-col divide-y self-start overflow-hidden rounded-xl border"
>
	<Command.Input
		class="shrink-0 focus-override h-input placeholder:text-foreground-alt/50 bg-background inline-flex truncate rounded-tl-xl rounded-tr-xl px-4 text-sm transition-colors focus:ring-0 focus:outline-hidden"
	/>
	<Command.List
		class="overflow-x-hidden overflow-y-auto px-2 pb-2"
	>
		<Command.Viewport>
			<Command.Empty
				class="text-muted-foreground flex w-full items-center justify-center pt-8 pb-6 text-sm"
				>://</Command.Empty
			>

			<Command.Group>
				<Command.GroupHeading
					class="text-muted-foreground px-3 pt-4 pb-2 text-xs"
				>
					Channels
				</Command.GroupHeading>
				<Command.GroupItems>
					{#each channels as ch}
						<Command.Item
							class="rounded-button data-selected:bg-muted flex h-10 cursor-pointer items-center gap-2 px-3 py-2.5 text-sm outline-hidden select-none"
						>
							{ch}
						</Command.Item>
					{/each}
				</Command.GroupItems>
			</Command.Group>
		</Command.Viewport>
	</Command.List>
</Command.Root>
