import {
	PUBLIC_DEVL_API,
	PUBLIC_DEVL_BASE,
	PUBLIC_DEVL_PROTO,
	PUBLIC_PROD_API,
	PUBLIC_PROD_BASE,
	PUBLIC_PROD_PROTO
} from "$env/static/public";

export const URLS = (): { api: string; base: string; proto: string } => {
	if (!import.meta.env.DEV) {
		return {
			api: PUBLIC_PROD_API,
			base: PUBLIC_PROD_BASE,
			proto: PUBLIC_PROD_PROTO
		};
	}

	return {
		api: PUBLIC_DEVL_API,
		base: PUBLIC_DEVL_BASE,
		proto: PUBLIC_DEVL_PROTO
	};
};
