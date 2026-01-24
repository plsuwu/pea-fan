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

//

export type Entry =
	| { _tag: "Channel"; data: ChannelEntry }
	| { _tag: "Chatter"; data: ChatterEntry };

export type Score =
	| { _tag: "Channel"; data: ChannelScore }
	| { _tag: "Chatter"; data: ChatterScore };

export { Result, Err, Ok } from "./result";
