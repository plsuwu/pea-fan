import { APP_TOKEN, CLIENT_ID } from '$env/static/private';

export const HELIX_API = 'https://api.twitch.tv/helix';
export const HELIX_USERS = `${HELIX_API}/users`;
export const HELIX_STREAMS = `${HELIX_API}/streams`;

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
        headers: headers,
    });
    
    const body = await res.json();
    if (res.status != 200) {
        console.error('error fetching from helix: ', res, body);
        return { status: res.status };
    }

    return { body, status: res.status };
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

function getBatchUrl(logins: string[]) {
	let uri = `${HELIX_USERS}?login=${logins.pop()}`;
	logins.forEach((element) => {
		uri += `&login=${element}`;
	});

	return uri;
}

function getAppTokenHeaders() {
	return {
		Authorization: `Bearer ${APP_TOKEN}`,
		'Client-Id': CLIENT_ID
	};
}
