// See https://svelte.dev/docs/kit/types#app.d.ts
//
import type { Logger } from "$lib/observability/instrumentation";
import type { Entry } from "$lib/types";
import "unplugin-icons/types/svelte";

// for information about these interfaces
declare global {
	namespace App {
		interface Error {
			message: string;
			code: number | string;
			traceId?: string;
			spanId?: string;
			details?: {
				type?: string;
				message?: string;
			};
		}

		interface Locals {
			// request
			channel: string | null;
			leaderboard: Entry | null;
			client: {
				userAgent: string;
				xforwarded: string;
				cfconnecting: string;
				xforwardedList: string[];
			};

            rateLimited: bool;

			// telemetry
			traceId?: string;
			spanId?: string;
			logger: Logger;
		}
		// interface PageData {}
		// interface PageState {}
		// interface Platform {}
	}
}

export {};
