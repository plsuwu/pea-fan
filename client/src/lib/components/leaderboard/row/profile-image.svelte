<script lang="ts">
	import { getAltImageSizeUrl } from "$lib/utils";
	import SkeletonImage from "./skeleton-image.svelte";

	let {
		element = $bindable(),
		image,
		alt,
		isLive = false,
		...rest
	}: {
		image: string;
		alt: string;
		element?: HTMLImageElement | undefined;
		isLive?: boolean;
		[key: string]: unknown;
	} = $props();
</script>

<div class="flex items-center transition-discrete duration-100 ease-out">
	<SkeletonImage
		bind:element
		src={getAltImageSizeUrl(image, "MD")}
		{alt}
		class="size-[30px] shrink-0 rounded-full transition-all
        duration-100 ease-out md:size-[55px] lg:size-[60px]"
		skeletonClass="lg:size-[60px] size-[35px]"
		{...rest}
	/>
</div>
