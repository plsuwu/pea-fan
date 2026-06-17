<script lang="ts">
	import type { PageProps } from "./$types";
	import peeLikerImage from "$lib/assets/img/bot_image.png";
	import { getAltImageSizeUrl, readableColor } from "$lib/utils";
	import { mode } from "mode-watcher";
	import Kbd from "$lib/components/misc/kbd.svelte";

	let { data }: PageProps = $props();

	let { botStatus } = $derived(data);
</script>

<svelte:head>
	<meta
		name="description"
		content="information about pee_liker and how to use his commands."
	/>
</svelte:head>

<div class="flex w-full flex-row justify-center">
	<div
		class="mt-10 flex flex-row items-center justify-center lg:w-2/3 xl:w-1/2 xl:max-w-[950px]"
	>
		<div class="flex flex-col justify-center">
			<div
				class="ml-5 flex w-full flex-row items-center space-x-8 self-center lg:ml-0"
			>
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
					<div class="text-base text-muted-foreground/85 lg:text-2xl">
						piss-related chatbot - currently DEAD
					</div>
				</div>
			</div>
			<div
				class="mt-10 flex min-w-[250px] list-inside flex-col items-start
            justify-center px-4 align-middle text-lg lg:px-4 xl:px-26 xl:text-2xl"
			>
				{#if botStatus.length === 0}
					<div
						class="mt-8 self-center text-base font-normal text-muted-foreground/30"
					>
						disabled everywhere for now :(
					</div>
				{/if}
				{#each botStatus as channel}
					<a
						href={`https://twitch.tv/${channel.login}`}
						target="_blank"
						rel="noopener noreferrer"
						class="flex w-full min-w-[300px] flex-row items-center justify-center
                        self-center transition-opacity duration-100 ease-in-out hover:opacity-50 md:self-start"
					>
						<div class="flex w-[250px] flex-row space-y-1 space-x-18">
							<div>
								<img
									src={getAltImageSizeUrl(channel.image, "SM")}
									alt={`${channel.login}`}
									class="size-[30px] rounded-full"
								/>
							</div>
							<div
								class="mt-px text-left"
								style:color={readableColor(channel.color, mode.current, 4.5)}
							>
								{channel.name}
							</div>
						</div>
					</a>
				{/each}
			</div>
			<div class="mt-14 xl:mt-22">
				<span class="text-[28px] font-extrabold line-through xl:text-5xl"
					><span class="text-muted-foreground/45">his usage instructions</span
					></span
				>
				<div class="mt-8 w-max text-xl line-through xl:ml-2">
					<Kbd class=" text-muted-foreground/45">
						!pisscount <span class="text-muted-foreground">[?username]</span>
					</Kbd>
				</div>
				<div
					class="mt-6 flex flex-row items-center align-middle text-lg line-through xl:ml-4"
				>
					<div>
						<p
							class="mb-2.5 text-base leading-[1.2] text-muted-foreground/45 lg:text-lg"
						>
							Responds with a specified user's pisscount.
						</p>
						<p
							class="mb-1 text-base leading-[1.2] text-muted-foreground/45 lg:text-lg"
						>
							If no user is specified, this command returns the pisscount for
							the requesting user instead.
						</p>
					</div>
				</div>
			</div>
			<div class="mt-18">
				<div class="text-center text-xl text-foreground lg:text-3xl">
					<div class="flex flex-col items-center justify-center">
						<img
							class="mb-4 size-20"
							src="https://static-cdn.jtvnw.net/emoticons/v2/emotesv2_790549147d4946b5b71db6345b5a2c05/static/light/3.0"
							alt="chocojDogSob"
						/>
						<div>they killed him</div>
					</div>
				</div>
			</div>
		</div>
	</div>
</div>
