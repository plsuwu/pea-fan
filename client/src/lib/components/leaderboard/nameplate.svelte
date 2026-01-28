<script lang="ts">
	import type { ChatterEntry, ChannelEntry } from "$lib/types";
	import { getAltImageSizeUrl } from "$lib/utils";
	import { rtUtil } from "$lib/utils/routing";

    let { entry }: { entry: ChatterEntry | ChannelEntry } = $props();
	let href = $derived(rtUtil.getTenantHref(entry.name).href);
	let src = $derived(getAltImageSizeUrl(entry.image, "MD"));
	let { color, login, name } = $derived(entry);

    const readableColor = (color: string): string => {
        return color === "#000000" ? "#A9A9A9" : color;
    }

</script>

<a
	{href}
	class="group flex flex-row items-center space-x-4 text-lg font-medium my-2"
>
	<img
		{src}
		alt={login}
		class="size-12 rounded-full outline-2 outline-offset-2
            outline-accent-foreground brightness-80 transition-discrete 
            duration-200 ease-in group-hover:brightness-100"
	/>
	<div
		style={`color: ${readableColor(color)};`}
		class="brightness-75 transition-discrete duration-200
        ease-in group-hover:brightness-100"
	>
		{name}
	</div>
</a>
