export interface Matchable<Tag extends string> {
	readonly _tag: Tag;
}

export type Patterns<T extends Matchable<string>, R> = {
	[K in T["_tag"]]: (value: Extract<T, { _tag: K }>) => R;
};

export function match<T extends Matchable<string>, R>(
	value: T,
	patterns: Patterns<T, R>
): R {
	const handler = patterns[value._tag as keyof typeof patterns];
	return handler(value as Parameters<typeof handler>[0]);
}


