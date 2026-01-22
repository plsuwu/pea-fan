import { Enum, type EnumType } from "./match";

export const TRACER_NAME = "client-tracer";

export type ChannelEntry = {
	id: string;
	name: string;
	login: string;
	color: string;
	image: string;
	ranking: number;
	total_chatter: number;
	total_channel: number;
	chatter_scores?: Array<_ChatterScore>;
};

export type ChatterEntry = {
	id: string;
	name: string;
	login: string;
	color: string;
	image: string;
	ranking: number;
	total: number;
	channel_scores?: Array<_ChannelScore>;
};

//

export type _ChannelScore = {
	channel_id: string;
	chatter_id: string;
	score: number;
	ranking: number;
	channel_login: string;
	channel_name: string;
	channel_color: string;
	channel_image: string;
};

export type _ChatterScore = {
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

export type EntryVariant = {
	Channel: ChannelEntry;
	Chatter: ChatterEntry;
};

const EntryEnum = Enum<EntryVariant>("Entry");
EntryEnum.variant("Channel");
EntryEnum.variant("Chatter");

export const Entry = EntryEnum.constructors();
export type Entry = EnumType<EntryVariant>;

export type ScoreVariants = {
	ChannelScore: _ChannelScore;
	ChatterScore: _ChatterScore;
};

const ScoreEnum = Enum<ScoreVariants>("Score");
ScoreEnum.variant("ChannelScore");
ScoreEnum.variant("ChatterScore");

export const Score = ScoreEnum.constructors();
export type Score = EnumType<ScoreVariants>;
export type ChannelScore = Extract<Score, { _tag: "ChannelScore" }>;
export type ChatterScore = Extract<Score, { _tag: "ChatterScore" }>;

export { Result, Ok, Err } from "./result";
export { type EnumType, Enum } from "./match";
