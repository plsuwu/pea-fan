export const PROTO = 'http';
export const PF_BASE_URL = 'localhost:5173';
export const PF_API_URL = 'api.localhost:5173';
// export const 


export const formatNumber = (n: number) => {
	const MAX_CHARS = 3;
	let num = (n + 1).toString();

	if (num.length < MAX_CHARS) {
		let ctr = MAX_CHARS - num.length;

		do {
			num = '&nbsp;' + num;
			ctr -= 1;
		} while (ctr > 0);
	}

	return num;
};

export const channels = [
	'sleepiebug',
	'parasi',
	'unipiu',
	'cchiko_',
	'liljuju',
	'vacu0usly',
	'bexvalentine',
	'rena_chuu',
	'snoozy',
	'womfyy',
	'kyoharuvt',
	'batatvideogames',
	'myrmidonvt',
	'kokopimento',
	'myramors',
	'sheriff_baiken',
	'lcolonq',
	'chocojax',
	'miffygeist',
	'haelpc',
	'gloomybyte',
	'niupao',
	'souly_ch',
	'kyundere',
	'miaelou',
	'saltae',
	'flippersphd',
	'misspeggyx'
];
