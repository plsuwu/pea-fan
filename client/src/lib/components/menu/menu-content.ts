import type { Component } from "svelte";
import { Compass, Info } from "@lucide/svelte";
import { URLS } from "$lib";

export type Content = { _tag: "content" };
export type Row = { _tag: "row" };
export type MenuRowContent = {
	target: "_self" | "_blank";
	href: string;
	title: string;
	shortcut?: string;
} & Content;

export type MenuRowItem = {
	_tag: "row";
	title: string;
	Icon?: Component;
	children?: (MenuRowContent | MenuRowItem)[];
} & Row;

export const left: MenuRowItem[] = [
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

export const right: MenuRowItem[] = [
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
