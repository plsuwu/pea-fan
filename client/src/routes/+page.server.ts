import type { PageServerLoad } from './$types';
import {
	API_HOSTNAME,
	BROWSER_CLIENT_ID,
	CHANNELS,
	GQL_QUERY_CHANNELDATA_BODY,
	ROOT_TWITCH_GQL_API,
	ROOT_TWITCH_HELIX_API,
	type ActiveSocketResponse,
	type GqlQuery
} from '$lib/client/constants';
import { APP_TOKEN } from '$env/static/private';

const API_URL = 'https://api.piss.fan';
const API_OPEN_WS = 'active-sockets';

export const load: PageServerLoad = async ({ locals }) => {
	const live = await getActive();
	const channel = locals.channel;

	if (!channel) {
		const channelDataArray = new Array();
		const channels = await Promise.all(
			CHANNELS.map(async (channel) => {
				let channelStatus = { channel, live: false, color: '#000000', total: 0 };
				if (live.active_broadcasters.includes(channel)) {
					channelStatus.live = true;
				}

				const channelData = await getChannelData(channel);
                channelStatus.total = channelData.total;
				return channelStatus;
			})
		);

		return {
			count: CHANNELS.length,
			channels,
			channelData: channelDataArray,
			color: null
		};
	}

	const channelData = await getChannelData(channel);
	const color = await getChannelColor(channel);
	return {
		count: 1,
		channels: [{ channel, live: live.active_broadcasters.includes(channel), total: channelData.total }],
		channelData,
		color
	};
};

const getActive = async (): Promise<ActiveSocketResponse> => {
	const uri = `${API_HOSTNAME}/${API_OPEN_WS}`;
	const res = await fetch(uri, {
		method: 'GET'
	});

	const body = await res.json();
	return body;
};

const getChannelData = async (channel: string) => {
	const uri = `https://${API_HOSTNAME}/ceilings/channel?name=${channel}`;
	const res = await fetch(uri, {
		method: 'GET'
	});

	let body = await res.json();
	return body;
};

const getChannelColor = async (channel: string) => {
	const userId = await getUserIdFromLogin(channel);
	const uri = `https://${ROOT_TWITCH_HELIX_API}/chat/color?user_id=${userId}`;

	const res = await fetch(uri, {
		method: 'GET',
		headers: {
			'client-id': '7jz14ixoeglm6aq8eott8196p4g5ox',
			authorization: `Bearer ${APP_TOKEN}`
		}
	});

	let { data } = await res.json();
	return data[0].color;
};

const getUserIdFromLogin = async (login: string) => {
	let body: GqlQuery = GQL_QUERY_CHANNELDATA_BODY;
	body.variables.channelLogin = login;

	const uri = `https://${ROOT_TWITCH_GQL_API}/gql`;
	const res = await fetch(uri, {
		method: 'POST',
		headers: {
			'client-id': BROWSER_CLIENT_ID
		},
		body: JSON.stringify(body)
	});

	const { data } = await res.json();
	return await data.channel.id;
};
