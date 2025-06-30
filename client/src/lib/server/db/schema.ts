import type { AnyPgColumn } from 'drizzle-orm/pg-core';
import {
	pgTable,
	text,
	bigint,
	timestamp,
	primaryKey,
	index,
	boolean,
    serial,
} from 'drizzle-orm/pg-core';

export const user = pgTable(
	'users',
	{
		id: serial('id').primaryKey().notNull(),
		userLogin: text('user_login').unique().notNull(),
		userId: text('user_id').notNull().unique(),
		color: text('color').default('#000000').notNull(),
		total: bigint({ mode: 'number' }).notNull().default(0),
		redact: boolean('redact').notNull().default(false),
		createdAt: timestamp('created_at').defaultNow(),
		updatedAt: timestamp('updated_at').defaultNow()
	},
	(t) => [index('idx_users_total').on(t.total.desc())]
);

export const channel = pgTable(
	'channels',
	{
		broadcasterLogin: text('broadcaster_login')
			.references((): AnyPgColumn => user.userLogin, {
				onDelete: 'cascade'
			})
			.primaryKey()
			.notNull(),
		broadcasterId: text('broadcaster_id')
			.references((): AnyPgColumn => user.userId, { onDelete: 'cascade' })
			.notNull(),
		profileImgUrl: text('profile_img_url').notNull(),
		total: bigint({ mode: 'number' }).default(0).notNull(),
		createdAt: timestamp('created_at').defaultNow(),
		updatedAt: timestamp('updated_at').defaultNow()
	},
	(t) => [index('idx_channel_total').on(t.total.desc())]
);

export const userChannelScore = pgTable(
	'user_channel_scores',
	{
		userLogin: text('user_login')
			.references((): AnyPgColumn => user.userLogin, {
				onDelete: 'cascade'
			})
			.notNull(),
		broadcasterLogin: text('broadcaster_login')
			.references((): AnyPgColumn => channel.broadcasterLogin, {
				onDelete: 'cascade'
			})
			.notNull(),
		score: bigint({ mode: 'number' }).notNull().default(0),
		createdAt: timestamp('created_at').defaultNow(),
		updatedAt: timestamp('updated_at').defaultNow()
	},
	(t) => [
		primaryKey({ columns: [t.userLogin, t.broadcasterLogin] }),
		index('idx_channel_user_scores_user_login').on(t.userLogin),
		index('idx_channel_user_scores_broadcaster_login').on(
			t.broadcasterLogin
		),
		index('idx_channel_user_scores_user_score').on(t.score.desc()),
		index('idx_channel_user_scores_channel_score').on(
			t.broadcasterLogin,
			t.score.desc()
		)
	]
);
