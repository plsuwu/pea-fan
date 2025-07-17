import type { AnyPgColumn } from 'drizzle-orm/pg-core';
import {
	pgTable as table,
	text,
	integer,
	timestamp,
	primaryKey,
	index,
	boolean
} from 'drizzle-orm/pg-core';

export const user = table(
	'users',
	{
		id: text('id').primaryKey().unique().notNull(),
		login: text('login').unique().notNull(),
		color: text('color').default('#000000').notNull(),
		image: text('image'),
		total: integer('total').default(1).notNull(),
		redact: boolean('redact').default(false).notNull(),
		createdAt: timestamp('created_at').defaultNow(),
		updatedAt: timestamp('updated_at').defaultNow()
	},
	(t) => [index('idx_users_total').on(t.total.desc())]
);

export const channel = table(
	'channels',
	{
		id: text('id')
			.references((): AnyPgColumn => user.id, { onDelete: 'cascade' })
			.primaryKey()
			.notNull(),
		total: integer('total').default(1).notNull(),
		createdAt: timestamp('created_at').defaultNow(),
		updatedAt: timestamp('updated_at').defaultNow()
	},
	(t) => [index('idx_channel_total').on(t.total.desc())]
);

export const score = table(
	'scores',
	{
		chatterId: text('chatter_id')
			.references((): AnyPgColumn => user.id, { onDelete: 'cascade' })
			.notNull(),
		broadcasterId: text('channel_id')
			.references((): AnyPgColumn => channel.id, { onDelete: 'cascade' })
			.notNull(),
		score: integer('score').notNull().default(1),
		createdAt: timestamp('created_at').defaultNow(),
		updatedAt: timestamp('updated_at').defaultNow()
	},
	(t) => [
		primaryKey({ columns: [t.chatterId, t.broadcasterId] }),
		index('idx_scores_chatter').on(t.chatterId),
		index('idx_scores_broadcaster').on(t.broadcasterId),
		index('idx_user_score').on(t.chatterId, t.score.desc()),
		index('idx_channel_score').on(t.broadcasterId, t.score.desc())
	]
);

// export const userChannelScore = pgTable(
// 	'user_channel_scores',
// 	{
// 		userLogin: text('user_login')
// 			.references((): AnyPgColumn => user.userLogin, {
// 				onDelete: 'cascade'
// 			})
// 			.notNull(),
// 		broadcasterLogin: text('broadcaster_login')
// 			.references((): AnyPgColumn => channel.broadcasterLogin, {
// 				onDelete: 'cascade'
// 			})
// 			.notNull(),
// 		score: integer({ mode: 'number' }).notNull().default(0),
// 		createdAt: timestamp('created_at').defaultNow(),
// 		updatedAt: timestamp('updated_at').defaultNow()
// 	},
// 	(t) => [
// 		primaryKey({ columns: [t.userLogin, t.broadcasterLogin] }),
// 		index('idx_channel_user_scores_user_login').on(t.userLogin),
// 		index('idx_channel_user_scores_broadcaster_login').on(
// 			t.broadcasterLogin
// 		),
// 		index('idx_channel_user_scores_user_score').on(t.score.desc()),
// 		index('idx_channel_user_scores_channel_score').on(
// 			t.broadcasterLogin,
// 			t.score.desc()
// 		)
// 	]
// );
