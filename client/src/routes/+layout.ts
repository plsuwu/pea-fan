import type { LayoutLoad } from "./$types";
import { browser } from "$app/environment";
import { type SystemModeValue, setMode } from "mode-watcher";

export const load: LayoutLoad = async ({ data }) => {
	if (browser) {
		const cookie = data.modePreference as SystemModeValue;
		setMode(cookie ?? "system");
	}

	return { ...data };
};
