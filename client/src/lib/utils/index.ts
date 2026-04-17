import { channelCache } from "$lib/observability/server/cache.svelte";
import {
	TWITCH_IMAGE_SIZE,
	type ChannelEntry,
	type ChannelScore,
	type ChatterEntry,
	type ChatterScore,
	type Entry,
	type PaginatedResponse,
	type TwitchImageSize,
} from "$lib/types";
import { sha256 } from "@oslojs/crypto/sha2";
import { encodeHexLowerCase } from "@oslojs/encoding";
import type { RequestEvent } from "@sveltejs/kit";
import { mode } from "mode-watcher";

export { readableColor } from "./color";

export type UntypedEntry<T = UntypedSubEntry> = {
	id: string;
	login: string;
	name: string;
	color: string;
	image: string;
	ranking: number;
	total: number;
	totalScores: number;
	scores: T[];
};

export type UntypedSubEntry = {
	channel_id: string;
	chatter_id: string;
	score: number;
	ranking: number;
	login: string;
	name: string;
	color: string;
	image: string;
};

export const reroutable = async (_: RequestEvent, channel: string) => {
	const isValid = await channelCache.exists(`#${channel}`);
	return isValid;
};

export const isLocalDomain = (host: string): boolean => {
	const parts = host.split(".");
	return (
		(parts.length === 1 && host.includes("localhost")) ||
		(parts.length === 2 && parts[1].split(":")[0] === "local")
	);
};

export const isIpAddr = (host: string): boolean => {
	const hostname = host.split(":")[0];

	const v4Addr = /^(\d{1,3}\.){3}\d{1,3}$/;
	const v6Addr = /^\[?([a-fA-F0-9:]+)\]?$/;

	return v4Addr.test(hostname) || v6Addr.test(hostname);
};

export const getBaseURLFromRequest = (host: string): string => {
	const parts = host.split(".");
	return parts.length > 1 ? parts.slice(1).join(".") : host;
};

export function isValidVariant(
	variant: string
): variant is "channel" | "chatter" {
	return variant === "channel" || variant === "chatter";
}

export function isChatterEntry(
	variant: Entry
): variant is { _tag: "Chatter"; data: ChatterEntry } {
	return variant._tag === "Chatter" ? true : false;
}

export function getRankColor(rank: number) {
	switch (rank) {
		case 1:
			return {
				color: "#FFCC33",
				thickness: 4,
			};

		case 2:
			return {
				color: "#E6F6FF",
				thickness: 4,
			};

		case 3:
			return {
				color: "#CC6600",
				thickness: 4,
			};
		default:
			return {
				color: mode.current === "dark" ? "#ffffff" : "#000000",
				thickness: 1,
			};
	}
}

export function isChannelEntry(
	variant: Entry
): variant is { _tag: "Channel"; data: ChannelEntry } {
	return variant._tag === "Chatter" ? true : false;
}

export function isNumeric(n: any): boolean {
	return !isNaN(parseFloat(n)) && isFinite(n);
}

export function strToNum(str: string): number | undefined {
	if (!isNumeric(str)) {
		return undefined;
	}

	return Math.abs(Number(str));
}

export function clamp(num: number, min = 0, max = num): number {
	if (num < min) return min;
	if (num > max) return max;
	return num;
}

export function getAltImageSizeUrl(url: string, size: TwitchImageSize): string {
	// i think this regex covers all possible variations, but i am not 100% on
	// this claim being reliable or even mostly true
	const suffixRegex = /-(\d+x\d+)\.(png|jpe?g)/;
	return url.replace(suffixRegex, `-${TWITCH_IMAGE_SIZE[size]}.$2`);
}

export function capitalize<T = string>(str: string): T {
	return (str.charAt(0).toUpperCase() + str.slice(1)) as T;
}

export function mapPagedResponseToEntries(
	leaderboard: PaginatedResponse,
	variant: "Chatter" | "Channel"
): Array<Entry> {
	return leaderboard.items.map(
		(item) =>
			({
				_tag: variant,
				data: {
					...item,
					image: getAltImageSizeUrl(item.image, "SM"),
				},
			}) as Entry
	);
}

export function intoUntypedSubEntries(
	subEntries: ChatterScore[] | ChannelScore[] | undefined,
	parent: "Channel" | "Chatter"
): UntypedSubEntry[] {
	if (!subEntries) return [];

	if (parent === "Channel") {
		const typed = subEntries as ChatterScore[];
		return typed.map((entry) => {
			return {
				chatter_id: entry.chatter_id,
				channel_id: entry.channel_id,
				score: entry.score,
				ranking: entry.ranking,
				login: entry.chatter_login,
				name: entry.chatter_name,
				color: entry.chatter_color,
				image: entry.chatter_image,
			};
		});
	} else {
		const typed = subEntries as ChannelScore[];
		return typed.map((entry) => {
			return {
				chatter_id: entry.chatter_id,
				channel_id: entry.channel_id,
				score: entry.score,
				ranking: entry.ranking,
				login: entry.channel_login,
				name: entry.channel_name,
				color: entry.channel_color,
				image: entry.channel_image,
			};
		});
	}
}

export function intoParentEntry(entry: UntypedSubEntry): UntypedEntry {
	return {
		id: entry.channel_id,
		login: entry.login,
		name: entry.name,
		color: entry.color,
		image: entry.image,
		ranking: entry.ranking,
		total: entry.score,
		totalScores: 0,
		scores: new Array(),
	};
}

export function intoUntypedEntry(entry: Entry): UntypedEntry {
	if (entry._tag === "Channel") {
		const typed = entry.data as ChannelEntry;
		const subEntries = intoUntypedSubEntries(typed.chatter_scores, entry._tag);
		return {
			id: typed.id,
			login: typed.login,
			name: typed.name,
			color: typed.color,
			image: typed.image,
			ranking: typed.ranking,
			total: typed.total_channel,
			scores: subEntries,
			totalScores: typed.total_scores,
		};
	} else {
		const typed = entry.data as ChatterEntry;
		const subEntries = intoUntypedSubEntries(typed.channel_scores, entry._tag);
		return {
			id: typed.id,
			login: typed.login,
			name: typed.name,
			color: typed.color,
			image: typed.image,
			ranking: typed.ranking,
			total: typed.total,
			totalScores: typed.total_scores,
			scores: subEntries,
		};
	}
}

export function getTokenHash(token: string) {
	const uint8Array = new TextEncoder().encode(token);
	const hash = sha256(uint8Array);

	return encodeHexLowerCase(hash);
}
