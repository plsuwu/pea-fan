import { buildHeadersAuthless } from "$lib/server/verify";
import { type Actions, fail } from "@sveltejs/kit";

function cleanInput(input: string | null) {
	if (!input || input == "") {
		return null;
	}
    
    return input.replaceAll(/[*_]/g, "");
}

export const actions = {
	contact: async ({ request, fetch, locals }) => {
		const logger = locals.logger.child({
			action: "contact",
		});

		try {
			const formData = await request.formData();

			const contentRaw = formData.get("content") as string;
			const emailRaw = formData.get("email") as string;
			const headers = buildHeadersAuthless(true);

			const res = await fetch("/api/contact", {
				method: "POST",
				headers,
				body: JSON.stringify({
					_client: locals.client.cfconnecting,
					data: {
						email: cleanInput(emailRaw),
						content: cleanInput(contentRaw),
					},
				}),
			});

			if (!res.ok) {
				logger.warn("failed to run action");
				return fail(res.status, {
					error: res.statusText,
				});
			}

			return { success: true, data: "ok" };
		} catch (err) {
			logger.error({ error: err }, "failure while running action");
			return fail(500, {
				error: err,
			});
		}
	},
} satisfies Actions;
