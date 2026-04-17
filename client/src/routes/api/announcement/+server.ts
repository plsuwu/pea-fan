import { json, type RequestEvent, type RequestHandler } from "@sveltejs/kit";
import { env } from "$env/dynamic/private";
import { Rh } from "$lib/utils/route";
import { logger } from "$lib/observability/server/logger.svelte";

const ADMIN_SESSION_TOKEN = env.ADMIN_SESSION_TOKEN;

export const DELETE: RequestHandler = async ({ locals, cookies }) => {
	const adminAnnouncement = new URL(`${Rh.apiAdmin}/announcement`);
    // ...
    
	return json({ status: 403, error: "forbidden" });
};

export const POST: RequestHandler = async ({ locals, cookies }) => {
	const adminAnnouncement = new URL(`${Rh.apiAdmin}/announcement`);
    // ...

	return json({ status: 403, error: "forbidden" });
};
