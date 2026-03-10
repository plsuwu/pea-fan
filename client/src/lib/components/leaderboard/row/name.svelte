<script lang="ts">
	const TRUNCATABLE_NAME_LENGTH = 16;

	let { name, color } = $props();

	function shouldTruncate() {
		return name.length > TRUNCATABLE_NAME_LENGTH;
	}

	let truncatedName = $derived.by(() => {
		if (shouldTruncate()) {
			return name.slice(0, TRUNCATABLE_NAME_LENGTH - 2);
		}

		return name;
	});
</script>

<div
	class="flex w-min max-w-min items-center justify-start text-2xl"
>
	{name}
</div>
