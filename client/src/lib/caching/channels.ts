import { Cache } from "$lib/caching";
import { util } from "$lib/caching/cache";

const DEFAULT_CHANNELS: string[] = new Array();

class ChannelCache extends Cache<string> {
	public async fetchData() {
		const res = await fetch(this.endpoint);

		if (!res.ok) {
			this.logger.warn({ response: res }, "failed to fetch channels");
			if (this.data.size > 0) { 
                return this.data;
            }

			return this.fallback;
		}

		const body = await res.json();
		return body.data;
	}

	public async read(): Promise<string[]> {
		await this.refresh();
		const cached = [...this.data];

		return cached;
	}
}

export const channelCache = new ChannelCache(
	"channel",
	util.endpoint("channel/all"),
	DEFAULT_CHANNELS
);
