import { beforeNavigate, afterNavigate } from "$app/navigation";

export const snapshot = {
	capture: () => {
		const main = document.querySelector("main");
		return main ? main.scrollTop.toString() : "0";
	},

	restore: (value: string) => {
		const main = document.querySelector("main");
		if (main) main.scrollTop = parseInt(value, 10);
	},
};
