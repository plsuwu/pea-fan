import { createClient, type RedisClientType } from 'redis';
import {
	helixFetchBatchedImages,
	helixFetchUserImage,
	helixGetIdsFromLogins
} from '../helix/utils';
import type {
	Chatter,
	Channel,
	CachedUserData,
	CacheRetrievalResult
} from '$lib/types';

export const USER_KEY_INDEX_DISPLAY_NAME = 0;
export const USER_KEY_INDEX_LOGIN = 1;
export const USER_KEY_INDEX_IMAGE = 2;
export const USER_KEY_INDEX_PREV_FETCH = 3;
export const USER_KEY_INDEX_REDACT = 4;
export const USER_KEY_INDEX_TOTAL = 5;
export const USER_KEY_INDEX_LEADERBOARD = 6;

export type KeyStoredType = 'user' | 'channel';
export type KeyReturnType = 'login' | 'broadcaster';

export interface UserTable {
	id: string;
	login: string;
	name: string;
	image: string;
	total: string;
	// leaderboard: { value: string; score: number }[];
}

interface RedisUserData {
	name: string;
	total: number;
	image?: string | null;
	redact: boolean;
	prevImgFetch: boolean;
}

interface FetchedImagesResult {
	login: string;
	image: string;
}

export class RedisClientPool {
	// we mirror the Twitch max in our application
	public readonly MAX_QUERY_LENGTH: number = 100;
	private readonly REDACTED_USER: string = '{{REDACTED_USER}}';

	private hostname: string;
	private port: number;
	private url: string;

	private client: RedisClientType;
	private connected: boolean = false;

	constructor(hostname?: string, port?: number) {
		this.hostname = hostname ?? 'localhost';
		this.port = port ?? 6380;

		this.url = `redis://${this.hostname}:${this.port}`;
		this.client = createClient({
			url: this.url
		});
	}

	/**
	 * Ensures client is connected to the Redis server
	 * @async
	 */
	private async connect() {
		if (!this.connected) {
			try {
				this.connected = true;
				await this.client.connect();
			} catch (err) {
				// avoids crashing if the socket is already open, which seems to happen but i don't know why.
				//
				// we probably want to check that the error is something like 'Socket already opened' but it refuses
				// to work for me and i dont know why, and frankly this should be fine.
				console.error('[?]: ', err);
				this.connected = true;
			}
		}
	}
	private getMigrationPairs(prefix: string, id: string, data: UserTable) {
		const { name, login, image, total } = data;
		return [
			{
				key: `${prefix}:${id}:name`,
				val: name
			},
			{
				key: `${prefix}:${id}:login`,
				val: login
			},
			{
				key: `${prefix}:${id}:image`,
				val: image
			},
			{
				key: `${prefix}:${id}:prev_fetch`,
				val: 1
			},
			// {
			// 	key: `${prefix}:${id}:redact`,
			// 	val: 0
			// },
			{
				key: `${prefix}:${id}:total`,
				val: total
			}
		];
	}

	public async migrateOldData() {
		this.connect();

		const rawUserKeys = await this.client.KEYS(`user:*:total`);
		const rawChannelKeys = await this.client.KEYS(`channel:*:total`);

		const userKeys = rawUserKeys.map((key) => key.split(':')[1]);
		const channelKeys = rawChannelKeys.map(
			(key) => key.split(':')[1].split('#')[1]
		);

		const userLoginIdMap = await Promise.all(
			(await helixGetIdsFromLogins(userKeys)).map(
				async ({ id, login, name, image }) => {
					try {
						const total =
							(await this.client.GET(`user:${name}:total`)) ??
							(await this.client.GET(`user:${login}:total`));
						let leaderboard = await this.client.zRangeWithScores(
							`user:${name}:leaderboard`,
							0,
							-1
						);

						if (leaderboard.length === 0) {
							leaderboard = await this.client.zRangeWithScores(
								`user:${login}:leaderboard`,
								0,
								-1
							);
						}

						const migration = this.getMigrationPairs('user', id, {
							id,
							login,
							name,
							image,
							total: total ?? '0'
						});

						for (const obj of migration) {
							const { key, val } = obj;

							// await this.client.DEL(key);
							await this.client.SET(key, String(val));
						}

						await this.client.del(`user:${id}:leaderboard`);
						await this.client.zAdd(
							`user:${id}:leaderboard`,
							leaderboard
						);

						return {
							id,
							total,
							leaderboard,
							login,
							name,
							image,
							prev_fetch: true,
						};
					} catch (err) {
						console.error('ERR @ USER:', name);
						console.error(`\t\t(id: ${id})`);
						console.error('REASON:', err);
					}
				}
			)
		);

		const userIds = userLoginIdMap.map((user) => user?.id ?? null).filter(Boolean);
        await this._migrate(userIds, 'user');

		const channelLoginIdMap = await Promise.all(
			(await helixGetIdsFromLogins(channelKeys)).map(
				async ({ id, login, name, image }) => {
					const total = await this.client.GET(
						`channel:#${login}:total`
					);
					const leaderboard = await this.client.zRangeWithScores(
						`channel:#${login}:leaderboard`,
						0,
						-1
					);

					const migration = this.getMigrationPairs(
						'channel',
						`#${login}`,
						{
							id,
							login,
							name,
							image,
							total: total ?? '0'
						}
					);

					for (const obj of migration) {
						const { key, val } = obj;
						await this.client.SET(key, String(val));
					}

					// this seems like something we will need
					await this.client.SET(`channel:#${login}:id`, id);
					await this.client.zAdd(
						`channel:#${login}:leaderboard`,
						leaderboard
					);

					return {
						id,
						total,
						leaderboard,
						login,
						name,
						image
					};
				}
			)
		);

		// await this._migrate(rawUserKeys, 'user');
		// await this._migrate(rawChannelKeys, 'channel');
	}

	private async _migrate(keys: string[], keyType: string) {
		await Promise.all(
			keys.map(async (id) => {
				const total = Number(
					await this.client.GET(`${keyType}:${id}:total`)
				);
				await this.client.zAdd(`${keyType}:global:leaderboard`, {
					value: id,
					score: total
				});
			})
		);
	}

	/**
	 * Retrieves chatter data across all channels.
	 * @async
	 */
	public async getChatters(
		_cursor: number = 0,
		_max: number = this.MAX_QUERY_LENGTH
	) {
		const users = await this.getLeaderboard<Chatter>(
			'user',
			'login',
			'global'
		);
		return users;
	}

	/**
	 * Retrieves all channel data.
	 * @async
	 */
	public async getChannels(
		_cursor: number = 0,
		_max: number = this.MAX_QUERY_LENGTH
	) {
		const leaderboard = await this.getLeaderboard<Channel>(
			'channel',
			'broadcaster',
			'global'
		);

		return leaderboard;
	}

	/**
	 * Retrieves a user's leaderboard (for both channels and chatters).
	 *
	 * todo: maybe refactor the way we handle user/channel paths in this function so we can make sure we dont accidentally let something
	 *       slip through the cracks? currently i think this is it for this implementation??
	 *
	 * @async
	 * @param storedKey The user type as defined in the cache: either `'user'` or `'channel'`.
	 * @param user The user's name/login - if `storedKey` is `channel`, this will automatically be prepended with `'#'`.
	 *  > `'global'` is used to fetch either the global channels or users leaderboard (e.g quering for `user:global:leaderboard`).
	 * @return Any associated leaderboard as a json-like structure in the format `{ name: [channel/login], score: [count] }`.
	 */
	public async getLeaderboard<T extends CachedUserData>(
		storedKey: KeyStoredType,
		returnKey: KeyReturnType,
		user: string,
		cursor: number = 0,
		max: number = this.MAX_QUERY_LENGTH
	) {
		// ensure open Redis connection
		this.connect();

		// euehuhgughuheurhrrrgh
		const key = `${storedKey}:${storedKey === 'channel' && user !== 'global' ? `#${user}` : user}:leaderboard`;
		const leaderboard = await this.client.zRangeWithScores(
			key,
			cursor,
			cursor + (max - 1),
			{
				REV: true
			}
		);

		const board = leaderboard.map((item) => {
			return {
				// channel names will still have the leading `#` connected (this could be corrected
				// in the Rust lexer) - if this split call is nullish then we have a chatter's login
				// and we set `name` to the value directly
				name: item.value.split('#')[1] ?? item.value,
				total: item.score
			};
		});

		if (storedKey !== 'channel') {
			const fetched = await this.getUserData<T>(
				storedKey,
				returnKey,
				board
			);

			return fetched;
		} else {
			return await Promise.all(
				board.map(async (broadcaster) => {
					// make sure we redact chatters that have requested
					// to be redacted!
					const redact = await this.client.GET(
						`user:${broadcaster.name}:redact`
					);

					return {
						[returnKey]: redact
							? this.REDACTED_USER
							: broadcaster.name,
						total: broadcaster.total,
						image: redact
							? null
							: await this.getChannelImage(broadcaster.name)
					};
				})
			);
		}
	}

	/**
	 * Retrieves user data for **all** users in the cache under a specified base key in Redis. Generally speaking,
	 * this function provides the primary user data fetching driver logic for the `RedisClientPool` class.
	 *
	 * Access is generally facilitated through helper methods (`getChannels` or `getChatters`).
	 * @async
	 * @param storedKey The user's type as defined in the cache: should be either `'user'` or `'channel'`.
	 * @param returnKey The user's type as defined by the application: should be either `'broadcaster'` or `'login'`.
	 * @return An array of chatters or channels alongside respective cached data.
	 */
	public async getUserData<T extends CachedUserData>(
		storedKey: KeyStoredType,
		returnKey: KeyReturnType,
		users: { name: string; total: number }[]
	): Promise<CacheRetrievalResult<T>> {
		// ensure we're connected to the cache
		this.connect();

		const needsCacheAttempt = new Array();
		const resolved = await Promise.all(
			users.map(async (user) => {
				let { name, total, image, redact, prevImgFetch } =
					await this.getUserFromCache(storedKey, user.name);

				// console.log(name, total, image, redact, prevImgFetch);

				// avoid trying to re-fetch user data for redacted users or users that no longer
				// have assiociated data (e.g banned users) by setting the `prev_helix_fetch` flag
				// in the cache
				if (
					storedKey === 'user' &&
					!image &&
					!prevImgFetch &&
					!redact &&
					name !== this.REDACTED_USER
				) {
					// we probably want to figure out a better way to determine if we've previously
					// tried to fetch a user's image and were unable to do so; this is ultimately a
					// bit hacky and doesn't seem particularly reliable :)
					this.client.SET(`user:${name}:prev_helix_fetch`, 1);
					needsCacheAttempt.push(name);
				}

				if (total) {
					return { [returnKey]: name, total, image } as unknown as T;
				}
			})
		);

		// if a user does not have their profile image cached yet, fetch their images in batched sets
		// from helix, then map over the resulting array to push each entry into Redis.
		//
		// this setup means the cached data won't yet be available to the clientside, and will require
		// another request to hydrate user avatar fields in the DOM; not ideal but this only needs to
		// happen once EVER and it avoids creating just an insane amount of remapping array types and
		// object structures.
		if (needsCacheAttempt.length > 0) {
			await Promise.all(
				(await this.fetchBatchedImages(needsCacheAttempt)).map(
					async (user) => {
						// in an ideal world, we tack the fetched images onto the `users` array prior to
						// caching the URLs in the following call
						//
						// given we dont care so much about this being IMMEDIATELY available in the cache,
						// i believe we should just be able to let the runtime just handle this call when
						// there is free processor time by omitting the `await`, no?
						this.cacheUserImage(user.login, user.image);
					}
				)
			);
		}
		// filter out nullish entries and sort the final array in descending order
		const result: T[] = resolved
			.filter((i) => i != null && i != undefined)
			.sort((a: T, b: T) => {
				return a.total < b.total ? 1 : -1;
			});

		return result;
	}

	/**
	 * Fetches user profile images from helix in a single batch job.
	 * @async
	 * @param logins The usernames for the users to have their profile image fetched.
	 * @return An array containing a user's `login` and their profile image (`image`, a url).
	 */
	private async fetchBatchedImages(
		logins: string[]
	): Promise<FetchedImagesResult[]> {
		let retrieved = new Array();
		let remaining = logins.length;
		let cursor = 0;
		while (remaining) {
			// maximum Helix query batch is 100 usernames/ids, so we split the passed `logins`
			// array into up to 100 items before performing a fetch call
			const arraySlice = logins.slice(
				cursor,
				Math.min(cursor + this.MAX_QUERY_LENGTH, logins.length)
			);

			// perform the fetch
			//
			// i really dont want to create an interface for each object in the `data`
			// array field so we just `any` this for now
			const fetched: { data: any[] } | null =
				await helixFetchBatchedImages(arraySlice);

			if (fetched) {
				const { data } = fetched;
				retrieved.push(...data);
			}

			// adjust the cursor and remaining element count by the number of items we
			// just queried
			//
			// todo: i am looking at this and i imagine we could just adjust these by
			// `arraySlice.length`, right?
			cursor += Math.min(this.MAX_QUERY_LENGTH, remaining);
			remaining -=
				remaining > this.MAX_QUERY_LENGTH
					? this.MAX_QUERY_LENGTH
					: remaining;
		}

		return retrieved.map((user) => {
			return {
				login: user.display_name,
				image: user.profile_image_url
			};
		});
	}

	/**
	 * Fetch user data from the cache
	 * @async
	 * @param key The base key in Redis: either 'user' or 'channel'
	 * @param rawUser The raw key queried: e.g `'user:example_username:total'`
	 * @return The user data stored in Redis: `name`, `total`, `image`, `redact`, and `prevImgFetch`.
	 */
	private async getUserFromCache(
		key: KeyStoredType,
		name: string
	): Promise<RedisUserData> {
		const total = Number(await this.client.GET(`${key}:${name}:total`));
		const redact = Boolean(await this.client.GET(`${key}:${name}:redact`));
		const prevImgFetch = Boolean(
			await this.client.GET(`user:${name}:prev_helix_fetch`)
		);

		// avoid performing a query if the `redact` flag is set
		const image = redact
			? null
			: await this.client.GET(`${key}:${name}:image`);

		return {
			name: redact ? this.REDACTED_USER : name,
			total,
			image: redact ? null : image,
			redact,
			prevImgFetch
		};
	}

	/**
	 * Retrieves a profile image for a channel.
	 *
	 * This is provided as a separate function as we don't need (or want) to batch fetch these.
	 * @async
	 * @param broadcaster The `login` for the channel's owner
	 * @return The channel's `profile_image_url` string
	 */
	private async getChannelImage(broadcaster: string): Promise<string | null> {
		try {
			const cachedImage = await this.client.GET(
				`user:${broadcaster}:image`
			);
			if (!cachedImage) {
				const { data } = await helixFetchUserImage(broadcaster);
				this.cacheUserImage(broadcaster, data[0].profile_image_url);

				return data[0].profile_image_url;
			}

			return cachedImage;
		} catch (err) {
			console.log('error while fetching user image for:', broadcaster);
			console.log(err, '\n\n');
			return null;
		}
	}

	/**
	 * Sets a user's profile image in the Redis cache
	 * @async
	 * @param login The `login` string of the user
	 * @param url The `profile_image_url` string for the user
	 */
	private async cacheUserImage(login: string, url: string) {
		const key = `user:${login}:image`;
		this.client.SET(key, url);
	}
}

const RedisClient = new RedisClientPool();
export default RedisClient;
