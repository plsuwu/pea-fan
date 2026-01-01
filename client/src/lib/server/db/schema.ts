import { eq, sql } from 'drizzle-orm';
import {
	pgTable as tb,
	primaryKey as pk,
	index as idx,
	uniqueIndex as udx,
	integer,
	varchar,
	boolean,
	timestamp
} from 'drizzle-orm/pg-core';

export const chatter = tb(
	'chatter',
	{
		id: varchar('id', { length: 16 }).primaryKey(),
		login: varchar('login', { length: 25 }).notNull(),
		name: varchar('name', { length: 25 }).notNull(),
		color: varchar('color', { length: 8 }).default('#000000').notNull(),
		image: varchar('image').notNull(),
		total: integer('total').default(0).notNull(),
		private: boolean('private').default(false).notNull(),
		createdAt: timestamp('created_at').defaultNow().notNull(),
		updatedAt: timestamp('updated_at').defaultNow().notNull()
	},
	(t) => [
		udx('idx_chatter_id').on(t.id),
		idx('idx_chatter_login').on(t.login),
		idx('idx_chatter_rank').on(t.total, t.createdAt)
	]
);

export const channel = tb(
	'channel',
	{
		id: varchar('id', { length: 16 }).references(() => chatter.id),
		total: integer('total').default(0).notNull(),
		createdAt: timestamp('created_at').defaultNow().notNull(),
		updatedAt: timestamp('updated_at').defaultNow().notNull()
	},
	(t) => [
		pk({
			columns: [t.id]
		}),
		idx('idx_channel_channel_id_user_id').on(t.id),
		idx('idx_channel_ranks').on(t.id, t.total.desc(), t.createdAt.asc())
	]
);

export const score = tb(
	'score',
	{
		chatterId: varchar('chatter_id', { length: 16 }).references(
			() => chatter.id
		),
		channelId: varchar('channel_id', { length: 16 }).references(
			() => channel.id
		),
		score: integer('score').default(0).notNull(),
		createdAt: timestamp('created_at').defaultNow().notNull(),
		updatedAt: timestamp('updated_at').defaultNow().notNull()
	},
	(t) => [
		pk({ columns: [t.chatterId, t.channelId] }),
		idx('idx_score_chatter_id_channel_id').on(t.chatterId, t.channelId),
		idx('idx_chatter_all_channel_ranks').on(
			t.chatterId,
			t.score.desc(),
			t.createdAt.asc()
		),
		idx('idx_channel_all_chatter_ranks').on(
			t.channelId,
			t.score.desc(),
			t.createdAt.asc()
		)
	]
);
