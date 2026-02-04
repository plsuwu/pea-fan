<script lang="ts">
	import type { Component } from "svelte";
	import { Compass, Info } from "@lucide/svelte";
	import * as M from "$lib/shadcn-components/ui/menubar";
	import { URLS } from "$lib";

	type Content = { _tag: "content" };
	type Row = { _tag: "row" };
	type MenuRowContent = {
		target: "_self" | "_blank";
		href: string;
		title: string;
		shortcut?: string;
	} & Content;

	type MenuRowItem = {
		_tag: "row";
		title: string;
		Icon?: Component;
		children?: (MenuRowContent | MenuRowItem)[];
	} & Row;

	const left: MenuRowItem[] = [
		{
			_tag: "row",
			title: "navigate",
			Icon: Compass,
			children: [
				{
					_tag: "content",
					target: "_self",
					title: "channels",
					href: `${URLS().proto}://${URLS().base}/leaderboard/channel`,
					shortcut: "<C-!>"
				},
				{
					_tag: "content",
					target: "_self",
					title: "chatters",
					href: `${URLS().proto}://${URLS().base}/leaderboard/chatter`,
					shortcut: "<C-@>"
				}
			]
		}
	];

	const right: MenuRowItem[] = [
		{
			_tag: "row",
			title: "about",
			Icon: Info,
			children: [
				{
					_tag: "content",
					title: "plsuwu @ github",
					href: "https://github.com/plsuwu",
					target: "_blank"
				},
				{
					_tag: "content",
					title: "plss @ twitch",
					href: "https://twitch.tv/plss",
					target: "_blank"
				}
			]
		}
	];
</script>

{#snippet MenuItem(
	{ title, Icon, children }: MenuRowItem,
	align: "start" | "end"
)}
	<M.Menu>
		<M.Trigger class="text-[14px] font-semibold">
			<div class="flex items-center space-x-1.5">
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
										<div class="text-[12px] text-medium italic text-muted-foreground">
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
	<M.Root class="mx-6">
		{#each left as nav}
			{@render MenuItem({ ...nav }, "start")}
		{/each}
	</M.Root>

	<M.Root class="mx-6">
		{#each right as nav}
			{@render MenuItem({ ...nav }, "end")}
		{/each}
	</M.Root>
</div>
