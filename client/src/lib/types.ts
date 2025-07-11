export interface CachedUserData {
	image: string;
	total: number;
}

export interface Chatter extends CachedUserData {
	login: string;
}

export interface Channel extends CachedUserData {
	broadcaster: string;
}

export type CacheRetrievalResult<T extends CachedUserData> = T[];
