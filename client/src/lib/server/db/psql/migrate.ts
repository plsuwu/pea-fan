import { createClient, type RedisClientType } from 'redis';
import { helixGetIdsFromLogins } from '$lib/server/helix/utils';

import { eq, desc, sql } from 'drizzle-orm';
import _ from 'lodash';
import { db } from '$lib/server/db/psql/index';
import { chatters, channels, scores } from '$lib/server/db/psql/schema';

class _RedisHandler {
	public readonly MAX_QUERY_LENGTH: number = 100;
	private readonly REDACTED_USER: string = '{{ REDACTED }}';

	private hostname: string;
	private port: number;
	private url: string;

	private connected: boolean = false;
	public client: RedisClientType;

	constructor(hostname?: string, port?: number) {
		this.hostname = hostname ?? 'localhost';
		this.port = port ?? 6380;

		this.url = `redis://${this.hostname}:${this.port}`;
		this.client = createClient({
			url: this.url
		});
	}

	public async connect() {
		if (!this.connected) {
			try {
				this.connected = true;
				await this.client.connect();
			} catch (e) {
				// assume this is that 'already connected' error
				this.connected = true;
				console.error(
					'Redis error while attempting to connect to server:'
				);
				console.error(e);
			}
		}
	}
}

/** This handler is only meant for migration */
const RedisHandler = new _RedisHandler();



export async function _migrateOldFormat() {
	RedisHandler.connect();

	await updateChannelKeys();
	await updateChatterKeys();

    await validate();
}

async function updateChatterKeys() {
	const keys = await Promise.all(
		(await RedisHandler.client.KEYS('user:*:total')).map(async (item) => {
			let usr = item.split(':')[1].toLowerCase();

			return {
				name: usr
			};
		})
	);

	const userData = await helixGetIdsFromLogins(keys.map((item) => item.name));
	await Promise.all(
		userData.map(async (item) => {
			// RedisHandler.client.SET(`channel:${item.login}:id`, item.id);

			const total = await RedisHandler.client.GET(
				`user:${item.name}:total`
			);

			await RedisHandler.client.SET(
				`chatter:${item.id}:login`,
				item.login
			);
			await RedisHandler.client.SET(
				`chatter:${item.id}:total`,
				Number(total)
			);

			await db
				.insert(chatters)
				.values({
					id: item.id as string,
					login: item.login as string,
					name: item.name as string,
					color: item.color as string,
					image: item.image as string,
					total: Number(total ?? 0)
				})
				.onConflictDoUpdate({
					target: chatters.id,
					set: {
						login: item.login as string,
						name: item.name as string,
						color: item.color as string,
						image: item.image as string,
						total: Number(total ?? 0),
						updatedAt: sql`NOW()`
					}
				});

			let leaderboard = await RedisHandler.client.zRangeWithScores(
				`user:${item.name}:leaderboard`,
				0,
				-1,
				{ REV: true }
			);

			if (leaderboard.length === 0) {
				leaderboard = await RedisHandler.client.zRangeWithScores(
					`user:${item.login}:leaderboard`,
					0,
					-1,
					{ REV: true }
				);

				if (leaderboard.length === 0) {
					console.error(
						`leaderboard for ${item.id}:${item.login} has length of zero.`
					);
					leaderboard = new Array();
				}
			}

			await RedisHandler.client.zAdd(
				`chatter:${item.id}:leaderboard`,
				leaderboard
			);

			for (const key of leaderboard) {
				const br = key.value.split('#')[1].split(':')[0];
				const brId = (await RedisHandler.client.GET(
					`channel:${br}:id`
				)) as string;

				await db
					.insert(scores)
					.values({
						chatterId: item.id,
						channelId: brId,
						score: key.score
					})
					.onConflictDoUpdate({
						target: [scores.chatterId, scores.channelId],
						set: {
							score: key.score,
							updatedAt: sql`NOW()`
						}
					});
			}
		})
	);
}

async function updateChannelKeys() {
	const keys = await Promise.all(
		(await RedisHandler.client.KEYS('channel:*:total')).map(
			async (item) => {
				let br = item.split('#')[1].split(':')[0].toLowerCase();

				return {
					name: br
				};
			}
		)
	);

	const broadcastersData = await helixGetIdsFromLogins(
		keys.map((key) => key.name)
	);
	await Promise.all(
		broadcastersData.map(async (item) => {
			await RedisHandler.client.SET(`channel:${item.login}:id`, item.id);
			const total = await RedisHandler.client.GET(
				`channel:#${item.login}:total`
			);

			// await RedisHandler.client.SET(`channel:${br}:total`, total ?? 0);
			// await RedisHandler.client.DEL(`channel:#${br}:total`);

			console.log(`channel:${item.login}:id`, item.id);
			console.log(`channel:${item.login}:color`, item.color);

			await db
				.insert(chatters)
				.values({
					id: item.id as string,
					login: item.login as string,
					name: item.name as string,
					color: item.color as string,
					image: item.image as string,

					// totals for a channel vs. a chatter are different
					// and unrelated values!
					total: 0
				})
				.onConflictDoUpdate({
					target: chatters.id,
					set: {
						login: item.login as string,
						name: item.name as string,
						color: item.color as string,
						image: item.image as string,
						updatedAt: sql`NOW()`
					}
				});

			console.log(item.id, item.name, item.login);

			await db
				.insert(channels)
				.values({
					id: item.id as string,
					total: Number(total ?? 0)
				})
				.onConflictDoUpdate({
					target: channels.id,
					set: {
						total: Number(total ?? 0),
						updatedAt: sql`NOW()`
					}
				});
		})
	);
}

async function validate() {
	console.log('\n ------- \n');
	console.log('all keys added to database and updated in redis');
	console.log('starting validation');
	console.log('\n ------- \n');

	const totalEntries = await db.$count(chatters);
	const randIndex = Math.floor(Math.random() * totalEntries);

	const chId = ((await RedisHandler.client.KEYS(`chatter:*:total`)) as any[])[
		randIndex
	].split(':')[1];
	const randChatterLogin = await RedisHandler.client.GET(
		`chatter:${chId}:login`
	);
	const randChatterTotal = await RedisHandler.client.GET(
		`chatter:${chId}:total`
	);
	const randChatterLeaderboard = await RedisHandler.client.zRangeWithScores(
		`chatter:${chId}:leaderboard`,
		0,
		-1,
		{ REV: true }
	);

	const randChatter = {
		id: chId,
		login: randChatterLogin,
		total: randChatterTotal,
		leaderboard: randChatterLeaderboard
	};

	console.log(
		`redis - chatter no. ${randIndex} (of ${totalEntries} total in db):`,
		randChatter
	);

	const { login, total } = (
		await db
			.select({ login: chatters.login, total: chatters.total })
			.from(chatters)
			.limit(1)
			.where(eq(chatters.id, chId))
	)[0];

	const leaderboardSubquery = db
		.select()
		.from(scores)
		.where(eq(scores.chatterId, chId))
		.as('leaderboard');

	const leaderboard = await db
		.with(leaderboardSubquery)
		.select({ score: leaderboardSubquery.score, login: chatters.login })
		.from(leaderboardSubquery)
		.innerJoin(chatters, eq(chatters.id, leaderboardSubquery.channelId))
		.orderBy(desc(leaderboardSubquery.score));

	const dbChatter = {
		id: chId,
		login,
		total: String(total),
		leaderboard: [
			{
				value: `#${leaderboard[0].login}`,
				score: leaderboard[0].score
			}
		]
	};

	console.log('psql - chatter:', dbChatter);
	if (_.isEqual(randChatter, dbChatter)) {
		console.log('migration looks ok');
	}
}
