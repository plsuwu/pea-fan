import { createClient } from 'redis';
import {
	getBatchUserImageFromHelix,
	getUserImageFromHelix
} from '../helix/utils';
import type { Chatter } from '$lib/types';

const client = createClient({
	url: 'redis://localhost:6380'
});

await client.connect();

export const getAllKeyTotals = async (
	key: 'user' | 'channel',
	type: 'broadcaster' | 'login'
) => {
	const needsCachedImage = new Array();

	const rawKeys = await client.KEYS(`${key}:*:total`);
	const chatters = await Promise.all(
		rawKeys.map(async (userKey) => {
			let name = userKey.split(':')[1];
			const prevFetch = await client.GET(`user:${name}:all_fetched`);
			const total = await client.GET(`${key}:${name}:total`);
			let image = await client.GET(`${key}:${name}:image`);

			// if a user has the 'redact' flag set (i.e, `user:[login]:redact` === 1),
			// anonymize them to clients
			if (key === 'user') {
				const redact = await client.GET(`${key}:${name}:redact`);
				if (redact && Number(redact) === 1) {
					name = '[ANONYMOUS_USER]';
					image = null;
				} else if (!image && !Number(prevFetch)) {
					// avoid trying to re-fetch user data for users no longer have
					// assiociated data (e.g banned users) by setting the `prevFetch` flag
					// in the cache
					setAttemptedFetch(name);
					needsCachedImage.push(name);
				}
			}

			if (total) {
				return { [type]: name, total: Number(total), image };
			}
		})
	);

	if (needsCachedImage.length > 0) {
		await Promise.all(
			(await getBatchedUserImages(needsCachedImage)).map(async (user) => {
				await cacheUserImage(user.login, user.image);
			})
		);
	}

	const result = chatters
		.filter((i) => i != null && i != undefined)
		.sort((a, b) => {
			return a!.total < b!.total ? 1 : -1;
		});

	return result;
};

export const getBatchedUserImages = async (logins: string[]) => {
	let retrieved = new Array();
	let remaining = logins.length;
	let cursor = 0;

	while (remaining) {
		const arraySlice = logins.slice(
			cursor,
			Math.min(cursor + 100, logins.length)
		);

		const fetched: { data: any[] } | null =
			await getBatchUserImageFromHelix(arraySlice);
		if (fetched) {
			const { data } = fetched;
			retrieved.push(...data);
		}

		cursor += Math.min(100, remaining);
		remaining -= remaining > 100 ? 100 : remaining;
	}

	const result: { login: string; image: string }[] = retrieved.map((user) => {
		return {
			login: user.display_name,
			image: user.profile_image_url
		};
	});

	return result;
};

export const getUserImage = async (broadcaster: string) => {
	const cachedImage = await client.GET(`user:${broadcaster}:image`);
	if (!cachedImage) {
		const { data } = await getUserImageFromHelix(broadcaster);
		cacheUserImage(broadcaster, data[0].profile_image_url);

		return data[0].profile_image_url;
	}

	return cachedImage;
};

export const getBatchedUserImage = async (chatters: Chatter[]) => {
	let needsImage = new Array();
	let chattersWithImage = new Array();

	await Promise.all(
		chatters.map(async (chatter) => {
			const cachedImage = await client.GET(`user:${chatter}:image`);
			if (!cachedImage) {
				needsImage.push(chatter);
			} else {
				chattersWithImage.push({
					...chatter,
					image: cachedImage
				});
			}
		})
	);

	if (needsImage.length > 0) {
		await getBatchUserImageFromHelix(needsImage);
	}

	return chattersWithImage;
};

export const setAttemptedFetch = async (login: string) => {
	client.SET(`user:${login}:all_fetched`, 1);
};

export const cacheUserImage = async (login: string, imageUrl: string) => {
	const key = `user:${login}:image`;
	client.SET(key, imageUrl);
};
