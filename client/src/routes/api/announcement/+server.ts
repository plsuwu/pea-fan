import { json, type RequestEvent, type RequestHandler } from "@sveltejs/kit";
import { env } from "$env/dynamic/private";
import { Rh } from "$lib/utils/route";
import { logger } from "$lib/observability/server/logger.svelte";

const ADMIN_SESSION_TOKEN = env.ADMIN_SESSION_TOKEN;
const ANNOUNCEMENT_ENDPOINT = new URL(`${Rh.apiBase}/admin/announcement`);

const buildHeaders = (token: string): Headers => {
	const headers = new Headers();
	headers.set("content-type", "application/json");
	headers.set("authorization", token);

	return headers;
};

export const POST: RequestHandler = async (event: RequestEvent) => {
	return json({ success: "" });
};

async function retrieveAnnouncement(event: RequestEvent) {}

async function createAnnouncement(event: RequestEvent) {
	const validated = await event.fetch("/api/verify-session", {
		method: "POST",
	});
}
