<script lang="ts">
	let { children, ...restProps } = $props();
</script>

<kbd
	class={`bg-background-alt h-5 w-5 text-xxs items-center justify-center dark:bg-foreground dark:text-background
    pointer-events-none
    flex gap-1 rounded-sm border px-1.5 font-mono font-medium opacity-100 shadow-[0px_2px_0px_0px_#59595b] select-none
    dark:border-[rgba(0,_0,_0,_0.10)] dark:shadow-[0px_2px_0px_0px_#B8B8B8] ${restProps.class}`}
>
	{@render children?.()}
</kbd>
