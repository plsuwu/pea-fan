import { APP_TOKEN, CLIENT_ID } from '$env/static/private';

export const HELIX_API = 'https://api.twitch.tv/helix/users';

export const getUserImageFromHelix = async (login: string) => {
	const uri = `${HELIX_API}?login=${login}`;
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

export const getBatchUserImageFromHelix = async (logins: string[]): Promise<{ data: any[] } | null> => {
	const uri = getBatchUrl(logins);
	const headers = getAppTokenHeaders();

	const res = await fetch(uri, {
		method: 'GET',
		headers: headers
	});

	const body = await res.json();
	if (res.status != 200) {
		console.error('error fetching (batched) from helix: ', res, body);
		return null;
	}
    
    console.log('performed a batch fetch: ', uri);
    console.log(body);
	return body;
};

function getBatchUrl(logins: string[]) {
	let uri = `${HELIX_API}?login=${logins.pop()}`;
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
