import {
	TWITCH_IMAGE_SIZE,
	type ChannelEntry,
	type ChannelScore,
	type ChatterEntry,
	type ChatterScore,
	type Entry,
	type PaginatedResponse,
	type TwitchImageSize
} from "$lib/types";

export type UntypedEntry = {
	login: string;
	name: string;
	color: string;
	image: string;
	ranking: number;
	total: number;
	totalScores: number;
	scores: UntypedSubEntry[];
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
		return;
	}

	return Number(str);
}

export function clamp(num: number, min = 0, max = num): number {
	if (num < min) return min;
	if (num > max) return max;
	return num;
}

export function getAltImageSizeUrl(
	url: string,
	size: TwitchImageSize,
	defaultSize = TWITCH_IMAGE_SIZE.XL
): string {
	return url.replace(defaultSize, TWITCH_IMAGE_SIZE[size]);
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
					image: getAltImageSizeUrl(item.image, "SM")
				}
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
				image: entry.chatter_image
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
				image: entry.channel_image
			};
		});
	}
}

export function readableColor(color: string) {
	if (color === "#000000") {
		return "#A0A0A0";
	}

	return color;
}

export function intoUntypedEntry(entry: Entry): UntypedEntry {
	if (entry._tag === "Channel") {
		const typed = entry.data as ChannelEntry;
		const subEntries = intoUntypedSubEntries(typed.chatter_scores, entry._tag);
		return {
			login: typed.login,
			name: typed.name,
			color: typed.color,
			image: typed.image,
			ranking: typed.ranking,
			total: typed.total_channel,
			scores: subEntries,
			totalScores: typed.total_scores
		};
	} else {
		const typed = entry.data as ChatterEntry;
		const subEntries = intoUntypedSubEntries(typed.channel_scores, entry._tag);
		return {
			login: typed.login,
			name: typed.name,
			color: typed.color,
			image: typed.image,
			ranking: typed.ranking,
			total: typed.total,
			totalScores: typed.total_scores,
			scores: subEntries
		};
	}
}
