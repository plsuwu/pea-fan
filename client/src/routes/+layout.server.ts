import { logger } from "$lib/observability";
import type { LayoutServerData } from "./$types";

export const load: LayoutServerLoad = async ({ event, locals }) => {
    logger.info("hello from layout server load");
    console.debug("asdf");

	return {};
};
