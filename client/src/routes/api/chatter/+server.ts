import { json, type RequestEvent, type RequestHandler } from "@sveltejs/kit";

export const POST: RequestHandler = async (event: RequestEvent) => {
	// const data = await event.request.json();
	// const { current, previous, score, comment, requestingClient } = data;
	//
	//
	// const now = new Date();
	//
	// const body = JSON.stringify({
	// 	content: JSON.stringify({
	// 		current,
	// 		previous,
	// 		score,
	// 		comment,
	// 		requestingClient,
	// 		now,
	// 	}),
	// });

	// const res = await fetch(WEBHOOK_URL, {
	// 	method: "POST",
	// 	body,
	// 	headers: buildHeadersAuthless(true),
	// });

	return json({ status: 400 }, { status: 400 });
};
