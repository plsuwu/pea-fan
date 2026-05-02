import { Cache } from "$lib/caching";
import { util } from "./cache";

export type Announcement = {
	hash: string | null;
	content: string | null;
	seen?: boolean;
};

const DEFAULT_ANNOUNCEMENT: Announcement = {
	hash: null,
	content: null,
};

export class AnnouncementCache extends Cache<Announcement> {
	public async fetchData() {
		const res = await fetch(this.endpoint);

		if (!res.ok) {
			this.logger.error({ response: res }, "failed to fetch announcement data");
			if (this.data.size > 0) { 
                return this.data;
            }

			return this.fallback;
		}

		const body = await res.json();
		return body.data;
	}

	public async read(): Promise<Announcement> {
		await this.refresh();
		const cached = [...this.data];

		return cached[cached.length - 1];
	}
}

export const announcementCache = new AnnouncementCache(
	"announcement",
	util.endpoint("announcement"),
	DEFAULT_ANNOUNCEMENT
);
