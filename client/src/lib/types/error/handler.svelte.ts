import type { LogLevel } from "$lib/observability";
import { randomUUIDv7 } from "bun";

export type UUIDv7 = `${string}-${string}-${string}-${string}-${string}`;
const UUID_V7_REGEX =
	/^[0-9a-f]{8}-[0-9a-f]{4}-7[0-9a-f]{3}-[89ab][0-9a-f]{3}-[0-9a-f]{12}$/i;

type SerializedError = {
	id?: string;
	name: string;
	message: string;
	code: number;
	level: LogLevel;
	details?: unknown;
};

function isUUIDv7(id: string): id is UUIDv7 {
	return UUID_V7_REGEX.test(id);
}

class _ErrorManager {
	#queue = $state<Array<SerializedError>>(new Array());

	enqueue(error: SerializedError) {
		if (!isUUIDv7(error.id!)) {
			error.id = randomUUIDv7();
		}

		this.#queue.push(error);
	}

	clear() {
		this.#queue = new Array();
	}

	get queue(): Array<SerializedError> {
		return this.#queue;
	}
}

export const ErrorManager = new _ErrorManager();
