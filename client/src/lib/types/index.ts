export type HelixUser = {
	id: string;
	login: string;
	display_name: string;
	profile_image_url: string;
	color: string;
	total: number;
	private: boolean;
};

export type ChannelEntry = {
	id: string;
	name: string;
	login: string;
	color: string;
	image: string;
	ranking: number;
	total_chatter: number;
	total_channel: number;
	chatter_scores?: Array<ChatterScore>;
	total_scores: number;
};

export type ChatterEntry = {
	id: string;
	name: string;
	login: string;
	color: string;
	image: string;
	ranking: number;
	total: number;
	channel_scores?: Array<ChannelScore>;
	total_scores: number;
};

//

export type ChannelScore = {
	channel_id: string;
	chatter_id: string;
	score: number;
	ranking: number;
	channel_login: string;
	channel_name: string;
	channel_color: string;
	channel_image: string;
};

export type ChatterScore = {
	channel_id: string;
	chatter_id: string;
	score: number;
	ranking: number;
	chatter_login: string;
	chatter_name: string;
	chatter_color: string;
	chatter_image: string;
};

export type ScoreWindows = {
	yesterday: number;
	prev_week: number;
	prev_month: number;
	prev_year: number;
	last_7_days: number;
	last_30_days: number;
};

export type PaginatedResponse<T = ChatterEntry | ChannelEntry> = {
	page: number;
	total_items: number;
	total_pages: number;
	page_size: number;
	items: Array<T>;
};

export type PaginationData = {
	currentPage: number;
	totalItems: number;
	itemsPerPage: number;
	totalPages: number;
};

export type PaginatedRequest = {
	limit: string;
	page: string;
	scoreLimit?: string;
	scorePage?: string;
};

//

export type EntryOption = ChannelEntry | ChatterEntry;

export type Entry =
	| { _tag: "Channel"; data: ChannelEntry }
	| { _tag: "Chatter"; data: ChatterEntry };

export type Score =
	| { _tag: "Channel"; data: ChannelScore }
	| { _tag: "Chatter"; data: ChatterScore };

export { Result, Err, Ok } from "./result";

export const TWITCH_IMAGE_SIZE = {
	XXL: "600x600",
	XL: "300x300",
	LG: "150x150",
	MD: "70x70",
	SM: "50x50",
	XS: "28x28",
} as const;

export const DEFAULT_ICON_SIZE = TWITCH_IMAGE_SIZE.XL;
export type TwitchImageSize = keyof typeof TWITCH_IMAGE_SIZE;
