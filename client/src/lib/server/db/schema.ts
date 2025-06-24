import type { AnyPgColumn } from 'drizzle-orm/pg-core';
import { pgTable, serial, text, integer, json } from 'drizzle-orm/pg-core';

export const user = pgTable('users', {
	id: text('id').primaryKey(),
	login: text('login'),
	color: text('color'),
	total: integer('total'),
	channels: json('leaderboard').$type<{ channel: string; total: number }>()
});

export const channel = pgTable('broadcasters', {
	id: text('id')
		.references((): AnyPgColumn => user.id)
		.primaryKey(),
	total: integer('total'),
	chatters: json('leaderboard').$type<{ chatter: string; total: number }>()
});
