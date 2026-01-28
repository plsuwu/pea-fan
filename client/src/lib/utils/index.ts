import {
	TWITCH_IMAGE_SIZE,
	type Entry,
	type PaginatedResponse,
	type TwitchImageSize
} from "$lib/types";

export function isValidVariant(
	variant: string
): variant is "channel" | "chatter" {
	return variant === "channel" || variant === "chatter";
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
