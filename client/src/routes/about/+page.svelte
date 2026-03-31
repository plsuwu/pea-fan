<script lang="ts">
	import Faq from "./sections/faq.svelte";
	import IncorrectScores from "./sections/incorrect-scores.svelte";
</script>

<div class="flex w-full flex-col items-center justify-center">
	<div class="w-full px-4 xl:w-[65%] xl:px-1">
		<h1 class="text-3xl font-bold xl:text-6xl" id="faq">faq</h1>
		<h2 class="text-base font-semibold text-muted-foreground xl:text-2xl">
			common questions or misconceptions i see
		</h2>
		<div class="mb-16 border-b border-muted-foreground/55"></div>
		<div class="w-full justify-self-center px-1 lg:px-10">
			<Faq />
		</div>
	</div>
	<div class="mt-20 w-full px-1 xl:w-[65%]">
		<h1 class="text-3xl font-bold xl:text-6xl" id="incorrect-scores">
			incorrect scores
		</h1>
		<h2 class="text-base font-semibold text-muted-foreground xl:text-2xl">
            sorgy
        </h2>
		<div class="mb-4 border-b border-muted-foreground/55"></div>
		<div class="w-full justify-self-center xl:w-[90%]">
			<IncorrectScores />
		</div>
	</div>
</div>
