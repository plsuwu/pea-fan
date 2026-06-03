import { json, type RequestHandler } from "@sveltejs/kit";
import { env } from "$env/dynamic/private";

const ADMIN_SESSION_TOKEN = env.ADMIN_SESSION_TOKEN;

// I think we are just doing this directly from cache objects but if it fucks up
// due to client/server route issues then we are going to make this request
// instead i think...
export const GET: RequestHandler = async () => {
	const data = { content: null, hash: null };
	return json({ status: 200, data });
};

export const DELETE: RequestHandler = async ({ locals, cookies }) => {
	// const adminAnnouncement = new URL(`${Rh.apiAdmin}/announcement`);
	// ...

	return json({ status: 403, error: "forbidden" });
};

export const POST: RequestHandler = async ({ locals, cookies }) => {
	// const adminAnnouncement = new URL(`${Rh.apiAdmin}/announcement`);
	// ...

	return json({ status: 403, error: "forbidden" });
};
