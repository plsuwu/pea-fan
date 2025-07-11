import type { AnyPgColumn } from 'drizzle-orm/pg-core';
import {
	pgTable as table,
	text,
	bigint,
	timestamp,
	primaryKey,
	index,
	boolean,
	serial
} from 'drizzle-orm/pg-core';

export const user = table(
	'users',
	{
		id: text('id').primaryKey().notNull(),
		login: text('login').unique().notNull(),
		color: text('color').default('#000000').notNull(),
		image: text('image'),
		total: bigint({ mode: 'number' }).default(1).notNull(),
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
			.references((): AnyPgColumn => user.id)
			.primaryKey()
			.notNull(),
		broadcaster: text('broadcaster')
			.references((): AnyPgColumn => user.login)
			.notNull(),
		total: bigint({ mode: 'number' }).default(0).notNull(),
		createdAt: timestamp('created_at').defaultNow(),
		updatedAt: timestamp('updated_at').defaultNow()
	},
	(t) => [index('idx_channel_total').on(t.total.desc())]
);

export const score = table(
	'scores',
	{
		chatter: text('chatter')
			.references((): AnyPgColumn => user.login)
			.notNull(),
		broadcaster: text('broadcaster')
			.references((): AnyPgColumn => channel.broadcaster)
			.notNull(),
		score: bigint({ mode: 'number' }).notNull().default(1),
		createdAt: timestamp('created_at').defaultNow(),
		updatedAt: timestamp('updated_at').defaultNow()
	},
	(t) => [
		primaryKey({ columns: [t.chatter, t.broadcaster] }),
		index('idx_scores_chatter').on(t.chatter),
		index('idx_scores_broadcaster').on(t.broadcaster),
		index('idx_user_score').on(t.chatter, t.score.desc()),
		index('idx_channel_score').on(t.broadcaster, t.score.desc())
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
// 		score: bigint({ mode: 'number' }).notNull().default(0),
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
