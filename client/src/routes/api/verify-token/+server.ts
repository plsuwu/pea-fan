import { json, type RequestEvent, type RequestHandler } from "@sveltejs/kit";
import { INTERNAL_POST_TOKEN } from "$env/static/private";
import { logger } from "$lib/observability/server/logger.svelte";
import { constantTimeEqual } from "@oslojs/crypto/subtle";
import { sha256 } from "@oslojs/crypto/sha2";
import { decodeHex } from "@oslojs/encoding";

function verify(token: string) {
	try {
		const left = decodeHex(token);
		const right = decodeHex(INTERNAL_POST_TOKEN);

		return constantTimeEqual(left, right);
	} catch (err) {
		logger.error(
			{ token: token, error: err },
			"[API] failed to create hash from token"
		);

		return false;
	}
}

export const POST: RequestHandler = async (event: RequestEvent) => {
	const { request } = event;
	const { token } = await request.json();

	const verified = verify(token);
	logger.info(
		{
			verified: verified
		},
		"[API] verification result"
	);

	return json({ verified });
};
