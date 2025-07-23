export interface CachedUserData {
	image: string;
	total: number;
}

export interface Chatter extends CachedUserData {
	id: string;
	name: string;
	login: string;
	redact: boolean;
	prevFetch: boolean;
}

export interface Channel extends CachedUserData {
	id: string;
    name: string;
    login: string;
}

export type CacheRetrievalResult<T extends CachedUserData> = T[];
