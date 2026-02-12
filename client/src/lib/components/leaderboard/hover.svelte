<script lang="ts">
	import * as Tt from "$lib/shadcn-components/ui/tooltip";
	import { getAltImageSizeUrl } from "$lib/utils";
	import { readableColor } from "$lib/utils";

	let { login, name, imgUrl, color, total } = $props();

	let href = $derived(`https://twitch.tv/${login}`);
</script>

<Tt.Root delayDuration={10}>
	<Tt.Trigger>
		<a {href} rel="noreferrer noopener" target="_blank">
			<img
				src={getAltImageSizeUrl(imgUrl, "XS")}
				alt={login}
				class="z-10 rounded-full bg-background ring-1 ring-offset-1"
			/>
		</a>
	</Tt.Trigger>
	<Tt.Content
		class="border border-accent-foreground bg-background font-iosevka text-foreground"
		arrowClasses="bg-background border-b border-r border-accent-foreground"
	>
		<div
			class="flex justify-between text-base font-semibold"
			style={`color: ${readableColor(color)};`}
		>
			{name}
		</div>
		<div class="font-normal">
			total: <span class="text-sm font-bold">{total}</span>
		</div>
	</Tt.Content>
</Tt.Root>
