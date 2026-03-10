export type SearchResult = {
	id: string;
	name: string;
	login: string;
	color: string;
	image: string;
	similarity_score: number;
	total: number;
    ranking: number;
};

export function debounce<
	T extends (signal: AbortSignal, ...args: any[]) => any,
>(fn: T, delay: number = 300) {
	let timeoutId: ReturnType<typeof setTimeout>;
	let controller: AbortController | null = null;

	return (
		...args: Parameters<T> extends [AbortSignal, ...infer Rest] ? Rest : never
	) => {
		clearTimeout(timeoutId);
		controller?.abort();

		timeoutId = setTimeout(() => {
			controller = new AbortController();
			fn(controller.signal, ...args);
		}, delay);
	};
}
