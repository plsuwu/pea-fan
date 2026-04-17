<script lang="ts">
	import { page } from "$app/state";
	import { cn } from "$lib/shadcn-components/utils";
	import { links } from "./links";
	let currentPage = $derived(page.url.pathname);
</script>

<div class="mt-1 flex w-full flex-row items-center justify-start space-x-4">
	{#each links as item}
		<div>
			<a
				href={item.href}
				class={cn(
					"underline transition-opacity duration-200 ease-in-out hover:opacity-50",
					currentPage === item.href
						? "text-accent-foreground"
						: "text-accent-foreground/55 "
				)}
			>
				{item.title}
			</a>
		</div>
	{/each}
</div>
