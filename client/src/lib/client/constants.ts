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
	'sleepiebug',
	'parasi',
	'unipiu',
	'cchiko_',
	'liljuju',
    'kyundere',
    'miaelou',
    'saltae',
    'haelpc',
    'misspeggyx',

	'vacu0usly',
	'bexvalentine',
    'rena_chuu',

    'snoozy',
    'gloomybyte',
    'miffygeist',
	'womfyy',
    'niupao',

    'myrmidonvt',
    'myramors',
    'kyoharuvt',
	'batatvideogames',
	'kokopimento',
	'sheriff_baiken',

	'lcolonq',
	'chocojax',
	'souly_ch',

	'flippersphd',
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
