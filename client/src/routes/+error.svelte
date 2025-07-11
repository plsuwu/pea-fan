<script lang="ts">
	import { page } from '$app/state';

	let error = page.error;
</script>

<div class="flex w-full flex-col items-center justify-center bg-red-600/10 m-32 rounded-lg">
	<div
		class="inline-flex w-full flex-col items-center font-bold my-24 font-display -mt-48"
	>
		<h1 class="text-8xl">what</h1>
		<h2 class="text-2xl">how.......</h2>
	</div>

    <div class="w-full inline-flex flex-col items-center my-8">
        <p>
            you caused some kind of error:
        </p>
        <div class="my-4 font-semibold flex flex-col p-12 bg-background rounded-xl">
            <code>{page.status} {"->"} [{error?.message.toLowerCase()}]</code>
        </div>

        <a href="/" class="underline hover:opacity-35 transition-opacity duration-150 ease-in-out">click here to leave</a>
    </div>
</div>
