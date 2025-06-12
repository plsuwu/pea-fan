<script lang="ts">
	import { page } from '$app/state';

	let err = $state();
	$effect(() => {
		if (page.error) {
			err = page.error;
		}
	});
</script>

<div>
	<div class="mb-12 text-lg font-medium">there was an error: {err}</div>
	<div>i'm genuinely not even sorry</div>
	<div>this site is still under construction and its like 4am</div>
	<a class="text-blue-500 transition-opacity duration-150 hover:opacity-40" href="/"
		>{'<--'} go home.</a
	>
</div>
