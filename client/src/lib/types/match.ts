type VariantConfig = {
	[variantName: string]: {} | void;
};

type EnumVariant<Tag extends string, Data> = Data extends void
	? { _tag: Tag }
	: { _tag: Tag } & Data;

export type EnumType<Config extends VariantConfig> = {
	[K in keyof Config]: EnumVariant<K & string, Config[K]>;
}[keyof Config];

type VariantConstructors<Config extends VariantConfig> = {
	[K in keyof Config]: Config[K] extends void
		? () => EnumVariant<K & string, Config[K]>
		: (data: Config[K]) => EnumVariant<K & string, Config[K]>;
};

type MatchPattern<T extends { _tag: string }, R> = {
	[K in T["_tag"]]: (value: Extract<T, { _tag: K }>) => R;
};

export function Enum<Config extends VariantConfig>(_name?: string) {
	type E = EnumType<Config>;

	const constructors = {} as VariantConstructors<Config>;
	const variantNames = new Set<string>();

	const addVariant = <K extends keyof Config>(variant: K) => {
		variantNames.add(variant as string);
		constructors[variant] = ((data?: Config[K]) => {
			if (data === undefined || data === null) {
				return { _tag: variant };
			}

			return { _tag: variant, ...data };
		}) as any;
	};

	return {
		variant: addVariant,
		constructors: () => constructors,

		match: <R>(value: E, patterns: MatchPattern<E, R>): R => {
			const tag = value._tag;
			if (!variantNames.has(tag)) {
				throw new Error(`unknown enum variant: '${tag}'`);
			}

			return (patterns as any)[tag](value);
		},

		matchPartial: <R>(
			value: E,
			patterns: Partial<MatchPattern<E, R>> & { _: (value: E) => R }
		): R => {
			const tag = value._tag;
			const handler = (patterns as any)[tag] || patterns._;
			return handler(value);
		},

		is: <K extends E["_tag"]>(variant: K) => {
			return (value: E): value is Extract<E, { _tag: K }> => {
				return value._tag === variant;
			};
		},

		map: <K extends E["_tag"], R>(
			value: E,
			variant: K,
			fn: (data: Extract<E, { _tag: K }>) => R
		): R | undefined => {
			if (value._tag === variant) {
				return fn(value as Extract<E, { _tag: K }>);
			}

			return undefined;
		}
	};
}
