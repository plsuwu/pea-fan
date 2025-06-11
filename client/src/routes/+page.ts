import type { PageLoad } from './$types';

const API_URL = 'https://api.piss.fan/ceilings/channel';

type LeaderboardEntry = [String, number];

interface ChannelResponse {
	err: boolean;
	err_msg: string;
	total: string;
	leaderboard: Array<LeaderboardEntry>;
}

export const load: PageLoad = async () => {
	const fans = [
		'sleepiebug',
		'cchiko_',
		'myrmidon',
		'lcolonq',
		'liljuju',
		'parasi',
		'snoozy',
		'vacu0usly',
		'womfyy',
		'kyoharuvt',
		'myramors',
		'batatvideogames',
		'chocojax'
	];

	const uri = `${API_URL}?name=${fans[0]}`;

	const req = await fetch(uri, {
		method: 'GET'
	});

	const body = await req.json();

	return {
		body,
		broadcaster: fans[0]
	};
};
