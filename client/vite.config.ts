import devtoolsJson from "vite-plugin-devtools-json";
import tailwindcss from "@tailwindcss/vite";
import { defineConfig } from "vitest/config";
import { playwright } from "@vitest/browser-playwright";
import { sveltekit } from "@sveltejs/kit/vite";

export default defineConfig(({ command }) => {
	const target = "es2022";
	return {
		define: {
			__VITE_COMMAND__: JSON.stringify(command),
			__BUILDING__: command === "build"
		},
		plugins: [tailwindcss(), sveltekit(), devtoolsJson()],
		optimizeDeps: {
			esbuildOptions: {
				target
			}
		},
		esbuild: {
			target
		},
		server: { port: 5173 },
		test: {
			expect: { requireAssertions: true },

			projects: [
				{
					extends: "./vite.config.ts",

					test: {
						name: "client",

						browser: {
							enabled: true,
							provider: playwright(),
							instances: [{ browser: "chromium", headless: true }]
						},

						include: ["src/**/*.svelte.{test,spec}.{js,ts}"],
						exclude: ["src/lib/server/**"]
					}
				},

				{
					extends: "./vite.config.ts",

					test: {
						name: "server",
						environment: "node",
						include: ["src/**/*.{test,spec}.{js,ts}"],
						exclude: ["src/**/*.svelte.{test,spec}.{js,ts}"]
					}
				}
			]
		}
	};
});
