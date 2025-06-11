import type { PageLoad } from './$types';

const API_URL = "https://api.piss.fan/ceilings/channel";

type LeaderboardEntry = [String, number];

interface ChannelResponse {
    err: boolean,
    err_msg: string,
    total: string,
    leaderboard: Array<LeaderboardEntry>
}

export const load: PageLoad = async ({ params }) => {
	const fans = [
		'cchiko_',
		'sleepiebug',
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
    
    const scores = fans.map(async (fan) => {
        const uri = `${API_URL}?name=${fan}`;
        const res = await fetch(uri, {
            method: 'GET',
        });

        const body: ChannelResponse = await res.json();
        if (body.err === true) {
            return null;
        }
    });

	return {
		fans
	};
};
