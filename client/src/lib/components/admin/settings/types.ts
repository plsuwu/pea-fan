export type HookInfo = {
	id: string;
	type: "stream.online" | "stream.offline";
	status: "enabled" | string;
	version: "1";
	cost: number | string;
	condition: {
		broadcaster_user_id: string;
	};
	created_at: string;
};
