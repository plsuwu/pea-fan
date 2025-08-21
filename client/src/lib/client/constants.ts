export type LeaderboardEntry = [String, number];
export interface ChannelResponse {
	err: boolean;
	err_msg: string;
	total: string;
	leaderboard: Array<LeaderboardEntry>;
}
export interface ActiveSocketResponse {
	active_count: number;
	active_broadcasters: string[];
}

export const CHANNELS = [
    "batatvideogames",
    "bexvalentine",
    "byebi",
    "cchiko_",
    "chocojax",
    "flippersphd",
    "gloomybyte",
    "haelpc",
    "hempievt",
    "imnoteds",
    "kokopimento",
    "krumroll",
    "kyoharuvt",
    "kyundere",
    "lcolonq",
    "liljuju",
    "miaelou",
    "miffygeist",
    "misspeggyx",
    "myramors",
    "myrmidonvt",
    "niupao",
    "noi_vt",
    "parasi",
    "pekoe_bunny",
    "rena_chuu",
    "saltae",
    "sheriff_baiken",
    "sleepiebug",
    "snoozy",
    "souly_ch",
    "unipiu",
    "vacu0usly",
    "walfas",
    "womfyy"
];

export interface GqlQuery {
	operationName: string;
	variables: {
		channelLogin: string;
	};
	extensions: {
		persistedQuery: {
			version: number;
			sha256Hash: string;
		};
	};
}

export const GQL_QUERY_CHANNELDATA_BODY: GqlQuery = {
	variables: {
		channelLogin: ''
	},
	extensions: {
		persistedQuery: {
			version: 1,
			sha256Hash: 'fa66abee26833eb414516b617bc3b051664e57ecc816704fce0b91344cae6ecd'
		}
	},
	operationName: 'Chat_ChannelData'
};

export const ROOT_TWITCH_HELIX_API = 'api.twitch.tv/helix';
export const ROOT_TWITCH_GQL_API = 'gql.twitch.tv';

export const BROWSER_CLIENT_ID = 'kimne78kx3ncx6brgo4mv6wki5h1ko';

// export const API_HOSTNAME = 'leaderboard_api';
export const API_HOSTNAME = 'api.piss.fan';

export const ROOT_HOSTNAME = 'piss.fan';
export const ROOT_SUBDOMAIN = 'piss';
// export const ROOT_HOSTNAME = 'localhost:5173';
// export const ROOT_SUBDOMAIN = 'localhost:5173';
