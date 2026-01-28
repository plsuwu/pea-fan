import type { LayoutServerLoad } from "./$types";
import { fetchUtil } from "$lib/utils/fetching";

export const load: LayoutServerLoad = async ({ locals, fetch }) => {
	if (locals.channel) {
		const leaderboard = await fetchUtil.fetchSingle(
			{ fetch },
			"channel",
			"login",
			locals.channel
		);

		return {
            leaderboard,
			channel: locals.channel
		};
	}

	return {
        leaderboard: null,
		channel: null
	};
};
