import { logger } from "$lib/observability/server/logger.svelte";
import type { Action } from "svelte/action";

export const actions = {
	default: async ({ cookies, request, fetch, getClientAddress }) => {
		const formData = await request.formData();
		const current = formData.get("current") as string;
        const previous = formData.get("previous") as string;
        const score = formData.get("score") as string;

        const body = JSON.stringify({ current, previous, score });
        console.log(body);

		const res = await fetch("/api/score-update-request", {
			method: "POST",
			body, 
			headers: {
				"content-type": "application/json",
			},
		});

		console.log(res);
	},
} satisfies Action;
