import { APP_TOKEN, CLIENT_ID } from '$env/static/private';

export const HELIX_API = 'https://api.twitch.tv/helix';
export const HELIX_USERS = `${HELIX_API}/users`;
export const HELIX_STREAMS = `${HELIX_API}/streams`;
export const HELIX_COLORS = `${HELIX_API}/chat/color`;

// todo: we probably want to cache this so we query our own cache and refresh the thumbnail
// every ~5 mins (or whatever the regeneration timer is i cant remember)
export const helixFetchStreamState = async (logins: string[]) => {
	let uri = `${HELIX_STREAMS}?user_login=${logins.pop()}`;
	if (logins.length > 0) {
		logins.forEach((login) => {
			uri += `&user_login=${login}`;
		});
	}
	const headers = getAppTokenHeaders();
	const res = await fetch(uri, {
		method: 'GET',
		headers: headers
	});

	const body = await res.json();
	if (res.status != 200) {
		console.error('error fetching from helix: ', res, body);
		return { status: res.status };
	}

	return { body, status: res.status };
};

export const helixGetIdsFromLogins = async (logins: string[]) => {
	console.log('input keyspace:', logins.length);
	let retrieved = new Array();
	let remaining = logins.length;
	let cursor = 0;

	while (remaining) {
		const arraySlice = logins.slice(
			cursor,
			Math.min(cursor + 100, logins.length)
		);

		const fetched: { data: any[] } | null =
			await helixFetchBatchedImages(arraySlice);

		if (fetched) {
			const { data } = fetched;
            
			const ids = data.map((item) => item.id);

            data.sort((a, b) => a.id > b.id ? 1 : -1);
			const colors = (await helixFetchUserColor(ids)).data.sort(
				(a: any, b: any) => (a.user_id > b.user_id ? 1 : -1)
			);

			const zipped = data.map((item, idx) => {
				if (item.display_name !== colors[idx].user_name) {
					console.log('DIFFERING ITEMS:', item);
					console.log(colors[idx]);
				}
				return {
					color: colors[idx].color,
					...item
				};
			});

			retrieved.push(...zipped);
		}

		cursor += Math.min(100, remaining);
		remaining -= remaining > 100 ? 100 : remaining;
	}

	const formatted = retrieved.map((user) => {
		return {
			id: user.id,
			login: user.login,
			name: user.display_name,
			image: user.profile_image_url,
			color: user.color
		};
	});

	console.log('retrieved keyspace:', formatted.length);
	return formatted;

	// return retrieved.map((user) => {
	// 	return {
	// 		login: user.display_name,
	// 		image: user.profile_image_url
	// 	};
	// });
};

export const helixFetchUserImage = async (login: string) => {
	const uri = `${HELIX_USERS}?login=${login}`;
	const headers = getAppTokenHeaders();
	const res = await fetch(uri, {
		method: 'GET',
		headers: headers
	});

	const body = await res.json();
	if (res.status != 200) {
		console.error('error fetching from helix: ', res, body);
		return;
	}

	return body;
};

export const helixFetchUserColor = async (
	logins: string[]
): Promise<{ data: any[] }> => {
	const uri = getBatchUrl(logins, HELIX_COLORS, 'user_id');
	const headers = getAppTokenHeaders();

	const res = await fetch(uri, {
		method: 'GET',
		headers
	});

	const body = await res.json();
	if (res.status != 200) {
		console.error('error fetching (batched) from helix: ', res, body);
		console.error('attempting fallback');

		const resBackup = await Promise.all(
			logins.map(async (id) => {
				const resFb = await fetch(`${HELIX_COLORS}?user_id=${id}`, {
					method: 'GET',
					headers
				});

				if (resFb.status != 200) {
					console.error(
						`fallback handler failed for user_id '${id}'`
					);
					console.error(resFb);
					return null;
				}

				const bodyFb = await resFb.json();
				const { data } = bodyFb;
				return [...data];
			})
		);

		const result = resBackup.flatMap((item) => item).filter(Boolean);
		return { data: result };
	}

	return body;
};

export const helixFetchBatchedImages = async (
	logins: string[]
): Promise<{ data: any[] } | null> => {
	const uri = getBatchUrl(logins);
	const headers = getAppTokenHeaders();

	const res = await fetch(uri, {
		method: 'GET',
		headers: headers
	});

	const body = await res.json();
	if (res.status != 200) {
		console.error('error fetching (batched) from helix: ', res, body);
		console.error('attempting fallback');
		return await helixIndividualFetch(logins);
	}

	return body;
};

const helixIndividualFetch = async (
	logins: string[]
): Promise<{ data: any[] } | null> => {
	const backup = await Promise.all(
		logins.map(async (login) => {
			const res = await helixFetchUserImage(login);
			if (!res) {
				console.error(`backup fetch handler failed for ${login}`);
				return null;
			}

			const { data } = res;
			return [...data];
		})
	);

	const result = backup.flatMap((item) => item).filter(Boolean);
	return { data: result };
};

function getBatchUrl(
	logins: string[],
	base_url: string = HELIX_USERS,
	userType: string = 'login'
) {
	let uri = `${base_url}?${userType}=${logins.pop()}`;
	logins.forEach((element) => {
		uri += `&${userType}=${element}`;
	});

	return uri;
}

function getAppTokenHeaders() {
	return {
		Authorization: `Bearer ${APP_TOKEN}`,
		'Client-Id': CLIENT_ID
	};
}
