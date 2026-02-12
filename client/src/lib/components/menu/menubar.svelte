<script lang="ts">
	import type { Component } from "svelte";
	import * as M from "$lib/shadcn-components/ui/menubar";
	import { tick } from "svelte";
	import * as Command from "$lib/shadcn-components/ui/command";
	import * as Popover from "$lib/shadcn-components/ui/popover";
	import { left, right, type MenuRowItem, type MenuRowContent } from "./menu-content";
</script>

{#snippet MenuItem(
	{ title, Icon, children }: MenuRowItem,
	align: "start" | "end"
)}
	<M.Menu>
		<M.Trigger class="rounded-non text-[14px] font-semibold">
			<div class="flex items-center space-x-1.5 rounded-none">
				<Icon size={16} />
				<!-- <span>{title}</span> -->
			</div>
		</M.Trigger>
		<M.Content {align}>
			{#if children && children.length > 0}
				{#if children[0]._tag === "content"}
					{#each children as child}
						{@const content = child as MenuRowContent}
						<a
							href={content.href}
							target={content.target}
							class="w-full text-end ring-0 focus:ring-0 focus:outline-none"
						>
							<M.Item class="w-full font-iosevka text-[14px]">
								<div class="flex w-full items-center justify-between">
									<div class="font-bold">{content.title}</div>
									{#if content.shortcut}
										<div
											class="text-medium text-[12px] text-muted-foreground italic"
										>
											{content.shortcut}
										</div>
									{/if}
								</div>
							</M.Item>
						</a>
					{/each}
				{:else}
					{#each children as child}
						{@const subrow = child as MenuRowItem}
						{@render SubMenuItem({ ...subrow })}
					{/each}
				{/if}
			{/if}
		</M.Content>
	</M.Menu>
{/snippet}

{#snippet SubMenuItem({ title, Icon, children }: MenuRowItem)}
	<M.Sub>
		<M.SubTrigger class="font-iosevka">
			<div>
				{#if Icon}
					<Icon size={16} />
				{/if}
				{title}
			</div>
		</M.SubTrigger>
		<M.SubContent>
			{#if children && children.length > 0}
				{#if children[0]._tag === "content"}
					{#each children as child}
						{@const content = child as MenuRowContent}
						<a href={content.href} target={content.target} class="font-iosevka">
							<M.Item class="font-iosevka">
								{content.title}
							</M.Item>
						</a>
					{/each}
				{/if}
			{/if}
		</M.SubContent>
	</M.Sub>
{/snippet}

<div class="flex w-full justify-between">
	<div>
		<M.Root class="mx-6">
			{#each left as nav}
				{@render MenuItem({ ...nav }, "start")}
			{/each}
		</M.Root>
		<div></div>
	</div>

	<M.Root class="mx-6">
		{#each right as nav}
			{@render MenuItem({ ...nav }, "end")}
		{/each}
	</M.Root>
</div>
