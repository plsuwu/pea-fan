import { Rh } from "$lib/utils/route";

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

export class SearchHandler {
	readonly apiUrl = new URL(`${Rh.proto}://${Rh.api}/search/by-login`);

	loading = $state(false);
	results = $state<SearchResult[]>([]);
	count = $state(0);

	current = $state("");
	previous = $state("");

	constructor() {}


	public async search(signal: AbortSignal, q: string) {
	}

	private createQueryUrl(login: string): URL {
		const url = this.apiUrl;
		url.searchParams.set("login", login);

		return url;
	}

	public clearSearch() {
		this.clearQuery();
		this.results = [];
	}

	public resultsEmpty(): boolean {
		return this.results.length <= 0;
	}

	public queryIsEmpty(): boolean {
		return this.current === "" && this.previous === this.current;
	}

	public clearQuery() {
		this.current = "";
		this.previous = "";
	}

}
