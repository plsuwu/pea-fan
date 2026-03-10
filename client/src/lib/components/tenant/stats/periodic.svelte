<script lang="ts">
	import * as Accordion from "$lib/shadcn-components/ui/accordion";
	import calendar from "dayjs/plugin/calendar";
	import dayjs, { type Dayjs } from "dayjs";

	dayjs.extend(calendar);

	let tsNow = dayjs();

	function getPeriod(period: "day" | "week" | "month" | "year") {
		const end = tsNow.startOf(period);
		const start = end.subtract(1, period);

		return { start, end };
	}

	const timestamps = {
		yesterday: getPeriod("day"),
		lastWeek: getPeriod("week"),
		lastMonth: getPeriod("month"),
		lastYear: getPeriod("year"),
		last7Days: tsNow.subtract(7, "day"),
		last30Days: tsNow.subtract(30, "day"),
	} as const;

	type Timestamp = keyof typeof timestamps;

	function getCalendarPeriod(ts: Timestamp) {
		if (ts === "yesterday") {
			return "yesterday";
		}

		if (ts === "last7Days" || ts === "last30Days") {
			const period = timestamps[ts] as Dayjs;
			const str = `since ${period.format("MMM D")}`.toLowerCase();
			return str;
		}

		const { start } = timestamps[ts] as { start: Dayjs; end: Dayjs };

		if (ts === "lastWeek") {
			return `last week`.toLowerCase();
		} else if (ts === "lastMonth") {
			return `in ${start.format("MMMM")}`.toLowerCase();
		} else {
			return `in ${start.format("YYYY")}`.toLowerCase();
		}
	}

	let { yesterday, prevWeek, prevMonth, prevYear, last7Days, last30Days } =
		$props();
</script>

{#snippet StatLine(title: string, stat: string | number)}
	<div
		class="flex items-center justify-between space-x-1 text-[13px] md:text-[15px] lg:text-[16px]"
	>
		<div
			class="flex w-1/2 flex-1 shrink-0 flex-row font-semibold text-accent-foreground/55"
		>
			<div>
				{title}:
			</div>
		</div>
		<div>{stat} times</div>
	</div>
{/snippet}

<div>
	<Accordion.Root type="single" class="w-full" >
		<Accordion.Item value="periodic-piss-stats">
			<Accordion.Trigger
				class="border-b border-b-muted rounded-none pt-2 pb-3 text-sm font-semibold text-accent-foreground/55 lg:text-lg"
				>periodic piss stats</Accordion.Trigger
			>
			<Accordion.Content class="rounded-b-lg border-r border-b border-l p-4">
				{@render StatLine(
					getCalendarPeriod("yesterday"),
					yesterday.toLocaleString()
				)}
				<div class="my-4"></div>
				{@render StatLine(
					getCalendarPeriod("last7Days"),
					last7Days.toLocaleString()
				)}
				{@render StatLine(
					getCalendarPeriod("last30Days"),
					last30Days.toLocaleString()
				)}
				<div class="my-4"></div>
				{@render StatLine(
					getCalendarPeriod("lastWeek"),
					prevWeek.toLocaleString()
				)}
				{@render StatLine(
					getCalendarPeriod("lastMonth"),
					prevMonth.toLocaleString()
				)}
				{@render StatLine(
					getCalendarPeriod("lastYear"),
					prevYear.toLocaleString()
				)}
			</Accordion.Content>
		</Accordion.Item>
	</Accordion.Root>
</div>
