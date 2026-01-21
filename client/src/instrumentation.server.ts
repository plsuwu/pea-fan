import { ObservabilityManager } from "$lib/observability";

// re. exposing OTEL for collection from clients:
//  https://opentelemetry.io/docs/languages/js/exporters/

const sdk = new ObservabilityManager();
sdk.start();

process.on("SIGTERM", () => {
	sdk
		.shutdown()
		?.catch(console.error)
		.finally(() => process.exit(0));
});
