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
	<div>this is a prototype frontend get OFF my case</div>
	<a class="text-blue-500 transition-opacity duration-150 hover:opacity-40" href="https://piss.fan"
		>{'<--'} go home.</a
	>
</div>
