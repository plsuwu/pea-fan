// See https://svelte.dev/docs/kit/types#app.d.ts
//
import type { Logger } from "$lib/observability/instrumentation";
import type { Entry } from "$lib/types";

// for information about these interfaces
declare global {
	namespace App {
		interface Error {
			message: string;
            code: number | string;
			traceId?: string;
		}
		interface Locals {
			channel: string | null;
            leaderboard: Entry | null;

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
