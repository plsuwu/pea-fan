<script lang="ts">
	import type { PageProps } from "./$types";
	import peeLikerImage from "$lib/assets/img/bot_image.png";
	import { getAltImageSizeUrl, readableColor } from "$lib/utils";
	import { mode } from "mode-watcher";
	import Kbd from "$lib/components/misc/kbd.svelte";

	let { data }: PageProps = $props();

	let { botChannels } = $derived(data);
</script>

<div class="flex w-full flex-row justify-center">
	<div
		class="mt-10 flex flex-row items-center justify-center lg:w-2/3 xl:w-1/2 xl:max-w-[950px]"
	>
		<div class="flex flex-col justify-center">
			<div class="flex w-full flex-row items-center space-x-8 self-center">
				<img src={peeLikerImage} alt="pee liker" class="rounded-full" />
				<div class="flex w-full flex-col">
					<div class="text-3xl font-extrabold xl:text-5xl">
						<a
							href="https://twitch.tv/pee_liker"
							target="_blank"
							class="text-[#ffc654] transition-discrete duration-200 ease-in-out
                            hover:brightness-50"
							rel="noopener noreferrer">pee_liker</a
						>
					</div>
					<div class="text-lg text-muted-foreground/85 lg:text-2xl">
						currently enabled for these broadcasters:
					</div>
				</div>
			</div>
			<ul
				class="mt-10 flex min-w-[250px] list-inside flex-col items-start
            justify-center px-4 align-middle text-lg lg:px-4 xl:px-26 xl:text-2xl"
			>
				{#each botChannels as channel}
					<a
						href={`https://twitch.tv/${channel.login}`}
						target="_blank"
						rel="noopener noreferrer"
						class="min-w-[300px] self-center transition-opacity duration-100
                        ease-in-out hover:opacity-50 md:self-start"
					>
						<li
							class="flex w-full flex-row items-center justify-between space-y-2 space-x-8"
						>
							<span
								style:color={readableColor(channel.color, mode.current, 4.5)}
								>{channel.name}</span
							>
							<div>
								<img
									src={getAltImageSizeUrl(channel.image, "SM")}
									alt={`${channel.login}`}
									class="size-8 rounded-full"
								/>
							</div>
						</li>
					</a>
				{/each}
			</ul>
			<div class="mt-10 xl:mt-48">
				<span class="text-3xl font-extrabold xl:text-5xl">how he is used</span>
				<div class="mt-8 w-max text-xl xl:ml-2">
					<Kbd>
						!pisscount <span class="text-muted-foreground">[?username]</span>
					</Kbd>
				</div>
				<div
					class="mt-6 flex flex-row items-center align-middle text-lg xl:ml-4"
				>
					<div>
						<p class="mb-2.5 text-base leading-[1.2] lg:text-lg">
							Responds with a specified user's pisscount.
						</p>
						<p class="mb-1 text-base leading-[1.2] lg:text-lg">
							If no user is specified, this command returns the pisscount for
							the requesting user instead.
						</p>
					</div>
				</div>
			</div>
			<div class="mt-16">
				<div class="text-sm text-muted-foreground lg:text-base">
					Responses are queued and sent in one-and-a-half-second intervals, with
					the queue holding a maximum of 16 messages before it begins dropping
					queries. The queue is shared across
					<span class="font-bold italic">all</span> channels above as per 
					<a
						href="https://dev.twitch.tv/docs/chat#twitch-chat-rate-limits"
						target="_blank"
						rel="noopener noreferrer"
						class="text-blue-600/75 transition-opacity duration-200 ease-in-out hover:opacity-50"
						>Twitch IRC rate limits</a
					>.
				</div>
			</div>
		</div>
	</div>
</div>
