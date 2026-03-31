import type { Action } from "svelte/action";

export const actions = {
	default: async ({ request, fetch, locals }) => {
		const formData = await request.formData();

		const current = formData.get("current") as string;
		const previous = formData.get("previous") as string;
		const score = formData.get("score") as string;
		const comment = formData.get("comment") as string;
		const requestingClient = locals.client.cfconnecting;

		const body = JSON.stringify({
			current,
			previous,
			score,
			comment,
			requestingClient,
		});
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
