// See https://svelte.dev/docs/kit/types#app.d.ts
import type { Logger } from "$lib/observability/server/logger";

// for information about these interfaces
declare global {
	namespace App {
		interface Error {
			message: string;
			traceId?: string;
		}
		interface Locals {
			channel: string;

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
