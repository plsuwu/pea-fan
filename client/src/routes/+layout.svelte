<script lang="ts">
	import '../app.css';

	let { children } = $props();
</script>

<div class="h-screen w-screen flex flex-row items-center justify-center -my-48">
	{@render children()}
</div>
