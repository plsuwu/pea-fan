import type { LogLevel, ApiError } from "./types";
import { browser } from "$app/environment";
import { traceHandler as tracer } from ".";

export class ObservableErrorBuilder {
	private error: ApiError & { logLevel?: LogLevel };
	private traceId?: string;
	private spanId?: string;

	constructor(code: string | number, message: string) {
		this.error = {
			code,
			message,
			details: {}
		};

		if (!browser) {
			import("./server/tracing").then((tracing) => {
				const ctx = tracing.traceHandler.context!;
				this.traceId = ctx.traceId;
				this.spanId = ctx.spanId;
			});
		}
	}

	displayMessage(message: string): this {
		this.error.displayMessage = message;
		return this;
	}

	silent(): this {
		this.error.silent = true;
		return this;
	}

	details(details: Record<string, unknown>): this {
		this.error.details = { ...this.error.details, ...details };
		return this;
	}

	level(level: LogLevel): this {
		this.error.logLevel = level;
		return this;
	}

	withTraceContext(): this {
		if (this.traceId) {
			this.error.details.traceId = this.traceId;
		}

		if (this.spanId) {
			this.error.details.spanId = this.spanId;
		}

		return this;
	}

	build(): ApiError {
		return this.error;
	}

	async buildAndLog(): Promise<ApiError> {
		const error = this.build();

		if (!error.silent) {
			if (browser) {
				const { clientLogger } = await import("./client/logger");
				const level = (this.error.logLevel ??
					"error") as keyof typeof clientLogger;

				(
					clientLogger?.[level] as unknown as (
						message: string,
						data?: Record<string, unknown>
					) => void
				)?.(error.message, {
					code: error.code,
					details: error.details
				});
			} else {
				const { logger } = await import("./server/logger.svelte");
				const level = this.error.logLevel || "error";

				const { spanId, traceId } = tracer.context;
				const childLogger = logger.child({ spanId, traceId });

				childLogger[level](
					{ code: error.code, details: error.details },
					error.message
				);
			}
		}

		return error;
	}
}

export function createError(
	code: string | number,
	message: string
): ObservableErrorBuilder {
	return new ObservableErrorBuilder(code, message);
}
