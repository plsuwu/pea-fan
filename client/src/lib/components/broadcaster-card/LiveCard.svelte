<script lang="ts">
	import { AspectRatio } from 'bits-ui';

	interface BroadcasterProp {
		id: string;
		user_id: string;
		user_login: string;
		user_name: string;
		game_id: string;
		game_name: string;
		type: string;
		title: string;
		viewer_count: number;
		started_at: string;
		language: string;
		thumbnail_url: string;
		tag_ids: undefined;
		tags: string[];
		is_mature: boolean;
		rank: number;
		image: string;
	}

	let { broadcasters }: { broadcasters: BroadcasterProp[] } = $props();

	function getStreamImageUrl(url: string) {
		const formatted = url
			.replace('{width}', '1280')
			.replace('{height}', '720');
		return formatted;
	}
</script>

<div class="px-10 font-bold text-2xl hidden xl:block absolute">
	<div class="fixed bg-background/80 w-full z-20 py-4">live channels</div>
</div>
<div class="flex w-full flex-col xl:items-end items-start px-4 pt-6">
	<div class="flex flex-col 3xl:grid w-full mx-4 2xl:grid-cols-2">
		{#each broadcasters as ch}
			<div class="flex w-full scale-[0.8] shrink-0 flex-col items-center">
				<div>
					<img
						src={getStreamImageUrl(ch.thumbnail_url)}
						class="rounded-15px"
						alt={`${ch.user_login} stream preview`}
					/>

					<!-- // -->
					<div class="mx-12 my-4 flex flex-row items-center w-full space-x-8">
						<div class="flex flex-row items-center space-x-4">
							<img
								src={ch.image}
								class="h-12 w-12 xl:w-16 xl:h-16 shrink-0 rounded-full"
								alt={`${ch.user_login} avatar`}
							/>
						</div>
						<!-- // -->
						<div class="my- flex flex-col justify-center space-x-4">
							<div class="flex flex-row items-center space-x-4">
								<div
									class="flex flex-row items-center justify-center text-xl"
								>
									<div
										class="mr-3 h-5 w-5 animate-pulse rounded-full bg-red-700"
									></div>
									{ch.user_name}
								</div>
								<div class="text-2xl">//</div>
								<div class="xl:text-base text-xs text-nowrap overflow-hidden">{ch.game_name}</div>
							</div>
							<div class="flex flex-row items-center space-x-4 w-max">
								<div class="text-foreground-alt brightness-75 text-xs xl:text-sm">
									{ch.title}
								</div>
							</div>
						</div>
					</div>
				</div>
			</div>
		{/each}
	</div>
</div>
