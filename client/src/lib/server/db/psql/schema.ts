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

export const chatters = table(
	'chatters',
	{
		id: text('id').primaryKey().unique().notNull(),
		login: text('login').unique().notNull(),
        name: text('name').notNull(),
		color: text('color').default('#000000').notNull(),
		image: text('image'),
		total: integer('total').default(1).notNull(),
		redact: boolean('redact').default(false).notNull(),
		createdAt: timestamp('created_at').defaultNow(),
		updatedAt: timestamp('updated_at').defaultNow()
	},
	(t) => [index('idx_chatters_total').on(t.total.desc())]
);

export const channels = table(
	'channels',
	{
		id: text('id')
			.references((): AnyPgColumn => chatters.id, { onDelete: 'cascade' })
			.primaryKey()
			.notNull(),
		total: integer('total').default(1).notNull(),
		createdAt: timestamp('created_at').defaultNow(),
		updatedAt: timestamp('updated_at').defaultNow()
	},
	(t) => [index('idx_channel_total').on(t.total.desc())]
);

export const scores = table(
	'scores',
	{
		chatterId: text('chatter_id')
			.references((): AnyPgColumn => chatters.id, { onDelete: 'cascade' })
			.notNull(),
		channelId: text('channel_id')
			.references((): AnyPgColumn => channels.id, { onDelete: 'cascade' })
			.notNull(),
		score: integer('score').notNull().default(1),
		createdAt: timestamp('created_at').defaultNow(),
		updatedAt: timestamp('updated_at').defaultNow()
	},
	(t) => [
		primaryKey({ columns: [t.chatterId, t.channelId] }),
		index('idx_scores_chatter').on(t.chatterId),
		index('idx_scores_broadcaster').on(t.channelId),
		index('idx_user_score').on(t.chatterId, t.score.desc()),
		index('idx_channel_score').on(t.channelId, t.score.desc())
	]
);
