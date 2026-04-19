<script lang="ts">
	import Faq from "./sections/faq.svelte";
</script>

<svelte:head>
	<meta name="description" content="about piss.fan and some contact options" />
</svelte:head>

<div class="flex w-full flex-col items-center justify-center">
	<div class="w-full px-4 xl:w-[65%] xl:px-1">
		<h1 class="text-3xl font-bold xl:text-6xl" id="faq">FAQ</h1>
		<div class="mb-16 border-b border-muted-foreground/55"></div>
		<div class="w-full justify-self-center px-1 lg:px-10">
			<Faq />
		</div>
	</div>

	<div class="w-full px-4 xl:w-[65%] xl:px-1">
		<h1 class="text-3xl font-bold xl:text-6xl" id="faq">contact</h1>
		<div class="mb-16 border-b border-muted-foreground/55"></div>
		<div class="w-full px-1 lg:px-10">
			<ul class="w-full list-inside list-disc text-xl">
				<li
					class="flex w-[40%] flex-row items-center justify-between self-center"
				>
					<span class="text-muted-foreground/75"
						>- i think my dms are open:</span
					>
					<a
						href="https://x.com/owoplsuwu"
						class="underline transition-discrete duration-200 ease-in-out hover:brightness-50"
						target="_blank"
						rel="noopener noreferrer">twitter</a
					>
				</li>
				<li
					class="flex w-[40%] flex-row items-center justify-between self-center"
				>
					<span class="text-muted-foreground/75"
						>- alternatively, send me an email:</span
					>
					<a
						href="mailto:admin@piss.fan"
						class="underline transition-discrete duration-200 ease-in-out hover:brightness-50"
						target="_blank">admin@piss.fan</a
					>
				</li>
			</ul>
		</div>
	</div>

	<!-- <div class="mt-20 w-full px-1 xl:w-[65%]"> -->
	<!-- 	<h1 class="text-3xl font-bold xl:text-6xl" id="incorrect-scores"> -->
	<!-- 		incorrect scores -->
	<!-- 	</h1> -->
	<!-- 	<h2 class="text-base font-semibold text-muted-foreground xl:text-2xl"> -->
	<!--            sorgy -->
	<!--        </h2> -->
	<!-- 	<div class="mb-4 border-b border-muted-foreground/55"></div> -->
	<!-- 	<div class="w-full justify-self-center xl:w-[90%]"> -->
	<!-- 		<IncorrectScores /> -->
	<!-- 	</div> -->
	<!-- </div> -->
</div>
