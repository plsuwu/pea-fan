import { ObservabilityManager } from "$lib/observability";
import { diag, DiagConsoleLogger, DiagLogLevel } from "@opentelemetry/api";
import { createAddHookMessageChannel } from "import-in-the-middle";
import { register } from "node:module";

// re. exposing OTEL for collection from clients:
//  https://opentelemetry.io/docs/languages/js/exporters/

const { registerOptions } = createAddHookMessageChannel();
register("import-in-the-middle/hook.mjs", import.meta.url, registerOptions);

diag.setLogger(new DiagConsoleLogger(), DiagLogLevel.VERBOSE);

const sdk = new ObservabilityManager();
sdk.start();

process.on("SIGTERM", () => {
	sdk
		.shutdown()
		?.catch(console.error)
		.finally(() => process.exit(0));
});
