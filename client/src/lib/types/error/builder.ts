import type { LogLevel } from "$lib/observability";
export type ApiError = {
	code: string | number;
	message: string;
	details: Record<string, unknown>;
	silent?: boolean;
	displayMessage?: string;
};

export class ErrorBuilder {
	private error: ApiError;

	constructor(code: string | number, message: string) {
		this.error = {
			code,
			message,
			details: {}
		};
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

	build(): ApiError {
		return this.error;
	}
}
