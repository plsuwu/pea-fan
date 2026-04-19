import { buildHeaders } from "$lib/server/verify";
import type { RequestHandler } from "@sveltejs/kit";
import { env } from "$env/dynamic/private";
import { Rh } from "$lib/utils/route";
import { json } from "@sveltejs/kit";
import { randomUUID } from "node:crypto";

const ADMIN_SESSION_TOKEN = env.ADMIN_SESSION_TOKEN;
const HOOKS_ENDPOINT = `${Rh.apiAdmin}/helix/hooks`;

type HookInfo = {
	id: string;
	type: "stream.online" | "stream.offline";
	status: "enabled" | string;
	version: "1";
	cost: number | string;
	condition: {
		broadcaster_user_id: string;
	};
	transport: {
		callback: string;
		method: string;
		secret: null;
	};
	created_at: string;
};

type Data = { data: [string, HookInfo][] };

const generateDummyHook = (): Data => {
	const id = randomUUID().toString();
	return {
		data: [
			[
				"liljuju",
				{
					condition: { broadcaster_user_id: "533612086" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.142333276Z",
					id: "08917274-41b2-4e8b-b3dd-e4bcd4a962e0",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.online",
					version: "1",
				},
			],
			[
				"gibbbons",
				{
					condition: { broadcaster_user_id: "51845736" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.144219889Z",
					id: "7120f5f5-f907-439a-8b57-48f5414ccb1e",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.online",
					version: "1",
				},
			],
			[
				"noi_vt",
				{
					condition: { broadcaster_user_id: "675393188" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.144729672Z",
					id: "219810cc-67a6-494c-a796-aec85a8f4767",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.online",
					version: "1",
				},
			],
			[
				"miaelou",
				{
					condition: { broadcaster_user_id: "605418870" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.145392229Z",
					id: "ec57b9de-bd53-492d-9f52-42e146652350",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.online",
					version: "1",
				},
			],
			[
				"flippersphd",
				{
					condition: { broadcaster_user_id: "130738371" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.145859529Z",
					id: "f9695669-a704-463c-802c-d56ae15764cd",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.online",
					version: "1",
				},
			],
			[
				"b0barley",
				{
					condition: { broadcaster_user_id: "600818743" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.147154987Z",
					id: "01f18f45-92cc-4a12-a5aa-0d58bc99c97a",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.online",
					version: "1",
				},
			],
			[
				"rena_chuu",
				{
					condition: { broadcaster_user_id: "759166226" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.152863247Z",
					id: "3df01e0f-2523-4589-9b61-ddd23a930208",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.online",
					version: "1",
				},
			],
			[
				"kyundere",
				{
					condition: { broadcaster_user_id: "141880295" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.154389056Z",
					id: "18d2ccfb-7577-4bfa-b33d-d5e8fa9e09ce",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.online",
					version: "1",
				},
			],
			[
				"tear",
				{
					condition: { broadcaster_user_id: "104925213" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.154459671Z",
					id: "343e3ede-382a-4a16-86a3-4ab7eda8ee19",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.online",
					version: "1",
				},
			],
			[
				"bibibiscuitch",
				{
					condition: { broadcaster_user_id: "1335538461" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.168472273Z",
					id: "dc2f001a-1657-4689-8e0f-0de230f076a6",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.offline",
					version: "1",
				},
			],
			[
				"niupao",
				{
					condition: { broadcaster_user_id: "512796146" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.170266127Z",
					id: "840b3bdc-40b9-4cc9-80b7-696db9323e1e",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.offline",
					version: "1",
				},
			],
			[
				"odessavt",
				{
					condition: { broadcaster_user_id: "844897177" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.173304601Z",
					id: "a2e2a0ca-1f5b-4304-849a-ebfb6782d8c1",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.offline",
					version: "1",
				},
			],
			[
				"gibbbons",
				{
					condition: { broadcaster_user_id: "51845736" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.173350828Z",
					id: "d653a6e9-c6d9-406f-bce4-96f9eacfdeb0",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.offline",
					version: "1",
				},
			],
			[
				"tini",
				{
					condition: { broadcaster_user_id: "122338258" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.180172121Z",
					id: "986003f9-e55e-4efe-84fd-99f882e713ba",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.offline",
					version: "1",
				},
			],
			[
				"souly_ch",
				{
					condition: { broadcaster_user_id: "94316536" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.180793984Z",
					id: "63126f6d-bada-45a3-999f-db892572e0e7",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.offline",
					version: "1",
				},
			],
			[
				"harupi",
				{
					condition: { broadcaster_user_id: "899965170" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.18145685Z",
					id: "a2a0aec6-c54b-4fc3-94c8-3e8a8d469a29",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.offline",
					version: "1",
				},
			],
			[
				"nanolather",
				{
					condition: { broadcaster_user_id: "31086482" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.182331302Z",
					id: "4fe19d7b-47a0-4def-a8ee-57f0901efbb2",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.offline",
					version: "1",
				},
			],
			[
				"plss",
				{
					condition: { broadcaster_user_id: "103033809" },
					cost: 0,
					created_at: "2026-04-09T16:25:23.182514925Z",
					id: "87416778-2fc1-4b66-99ad-b7b60d432854",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.offline",
					version: "1",
				},
			],
			[
				"chocojax",
				{
					condition: { broadcaster_user_id: "26189911" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.182760883Z",
					id: "43e4d7b6-4d91-471f-a68e-8dda313ee952",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.offline",
					version: "1",
				},
			],
			[
				"kokopimento",
				{
					condition: { broadcaster_user_id: "24714810" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.183391905Z",
					id: "a7758c11-0940-4a02-a46b-054536a1faba",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.offline",
					version: "1",
				},
			],
			[
				"meiya",
				{
					condition: { broadcaster_user_id: "89007125" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.187214641Z",
					id: "94bff97e-f5b0-47ab-a06f-5a00b353045f",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.offline",
					version: "1",
				},
			],
			[
				"snoozy",
				{
					condition: { broadcaster_user_id: "446955795" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.190297308Z",
					id: "bb564657-0127-4145-a1c8-da7749c233af",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.offline",
					version: "1",
				},
			],
			[
				"flippersphd",
				{
					condition: { broadcaster_user_id: "130738371" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.192128239Z",
					id: "bedbb470-5aa2-40d8-8a10-5138d4e5ed64",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.offline",
					version: "1",
				},
			],
			[
				"miaelou",
				{
					condition: { broadcaster_user_id: "605418870" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.194387674Z",
					id: "48e92690-8898-4a42-b99d-8808079bee45",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.offline",
					version: "1",
				},
			],
			[
				"chikogaki",
				{
					condition: { broadcaster_user_id: "413015060" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.195797915Z",
					id: "46c56d02-0f4b-4de8-9bf1-30dd98e0e59a",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.online",
					version: "1",
				},
			],
			[
				"rena_chuu",
				{
					condition: { broadcaster_user_id: "759166226" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.195887556Z",
					id: "874b6631-6af9-459a-9df3-9e1347e8924a",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.offline",
					version: "1",
				},
			],
			[
				"kyoharuvt",
				{
					condition: { broadcaster_user_id: "741293014" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.196030035Z",
					id: "c4d995eb-251e-48a1-864e-1a89f7c68e44",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.offline",
					version: "1",
				},
			],
			[
				"liljuju",
				{
					condition: { broadcaster_user_id: "533612086" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.196911262Z",
					id: "e444a566-1a1a-410a-940e-3fecf9556d62",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.offline",
					version: "1",
				},
			],
			[
				"b0barley",
				{
					condition: { broadcaster_user_id: "600818743" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.197100653Z",
					id: "021046de-26ea-4355-bf2a-8752b6ab0a34",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.offline",
					version: "1",
				},
			],
			[
				"madmad01",
				{
					condition: { broadcaster_user_id: "864287979" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.197356801Z",
					id: "7f64cbc8-2d07-42ad-a29a-8e87d98267d5",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.offline",
					version: "1",
				},
			],
			[
				"tear",
				{
					condition: { broadcaster_user_id: "104925213" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.19765242Z",
					id: "8f610848-4f04-4f4b-969f-dc09b021ac4e",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.offline",
					version: "1",
				},
			],
			[
				"netuserhael",
				{
					condition: { broadcaster_user_id: "592547707" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.199265824Z",
					id: "54f854aa-bba6-43c3-88ca-3adc5bdb7115",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.offline",
					version: "1",
				},
			],
			[
				"womfyy",
				{
					condition: { broadcaster_user_id: "263446776" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.202307244Z",
					id: "74ae5b85-83e7-473b-9c8f-3549c08df62e",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.offline",
					version: "1",
				},
			],
			[
				"pachi",
				{
					condition: { broadcaster_user_id: "48807896" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.202529326Z",
					id: "ed66dec1-8f70-444a-974f-0343e566a13e",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.offline",
					version: "1",
				},
			],
			[
				"hempie",
				{
					condition: { broadcaster_user_id: "172265161" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.203407524Z",
					id: "81fc5584-0a12-4762-8ca0-06c56d71842d",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.offline",
					version: "1",
				},
			],
			[
				"vacu0usly",
				{
					condition: { broadcaster_user_id: "54833441" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.204747253Z",
					id: "22ffaf2b-70d7-48a9-8865-59b20e2dbc8e",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.offline",
					version: "1",
				},
			],
			[
				"kumomomomomomomo",
				{
					condition: { broadcaster_user_id: "786298312" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.20662657Z",
					id: "c73608d4-bfd1-4162-90fe-a323c4e15b1f",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.offline",
					version: "1",
				},
			],
			[
				"unipiu",
				{
					condition: { broadcaster_user_id: "874233986" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.20805163Z",
					id: "a9df2f86-f607-4137-8714-af3ec6c5b074",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.offline",
					version: "1",
				},
			],
			[
				"sleepiebug",
				{
					condition: { broadcaster_user_id: "610533290" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.209459953Z",
					id: "bac9b13b-5761-4143-9f1e-e830a83b6799",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.offline",
					version: "1",
				},
			],
			[
				"substituber",
				{
					condition: { broadcaster_user_id: "997068289" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.209586048Z",
					id: "b5052b07-28f8-416c-a97e-3074a5a7e0da",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.offline",
					version: "1",
				},
			],
			[
				"milia",
				{
					condition: { broadcaster_user_id: "188503312" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.210817983Z",
					id: "12bd5ec0-33c4-4ded-81e6-229211c99c73",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.offline",
					version: "1",
				},
			],
			[
				"baikenvt",
				{
					condition: { broadcaster_user_id: "62127668" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.211390291Z",
					id: "00eea812-5388-4526-8d7d-e846ab89bbd6",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.online",
					version: "1",
				},
			],
			[
				"myrmidonvt",
				{
					condition: { broadcaster_user_id: "83255335" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.213109657Z",
					id: "3b73bba6-7731-40ab-b670-1837d8402978",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.offline",
					version: "1",
				},
			],
			[
				"lcolonq",
				{
					condition: { broadcaster_user_id: "866686220" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.213505265Z",
					id: "f8f68042-19a5-4bd0-a4eb-d090e02a53ab",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.online",
					version: "1",
				},
			],
			[
				"kyundere",
				{
					condition: { broadcaster_user_id: "141880295" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.213658637Z",
					id: "e42437a8-cad5-4ef7-9c1d-366865b8c28e",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.offline",
					version: "1",
				},
			],
			[
				"misspeggyx",
				{
					condition: { broadcaster_user_id: "818067359" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.21411368Z",
					id: "576c2acb-2ce4-4238-8bde-2d2bd850ec25",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.online",
					version: "1",
				},
			],
			[
				"batatvideogames",
				{
					condition: { broadcaster_user_id: "539128874" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.214193979Z",
					id: "2aeb766f-c379-4fff-b39c-b351a51b490a",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.online",
					version: "1",
				},
			],
			[
				"noi_vt",
				{
					condition: { broadcaster_user_id: "675393188" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.216213502Z",
					id: "90c51a43-133f-4236-a6d1-676d8f6af763",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.offline",
					version: "1",
				},
			],
			[
				"parasi",
				{
					condition: { broadcaster_user_id: "834137500" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.218103258Z",
					id: "c72f3bf4-f716-4299-80b3-6b360d18e826",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.online",
					version: "1",
				},
			],
			[
				"aaallycat",
				{
					condition: { broadcaster_user_id: "276477565" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.218494061Z",
					id: "1ee10a82-29fa-4151-915c-8aff04b645fb",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.offline",
					version: "1",
				},
			],
			[
				"krumroll",
				{
					condition: { broadcaster_user_id: "782458136" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.219650885Z",
					id: "f108e36b-c3f9-4322-8c9a-6f1990cefa80",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.online",
					version: "1",
				},
			],
			[
				"miffygeist",
				{
					condition: { broadcaster_user_id: "795478771" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.220317752Z",
					id: "da84db7a-dc9d-44b7-8767-dafa806fbc31",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.offline",
					version: "1",
				},
			],
			[
				"myramors",
				{
					condition: { broadcaster_user_id: "478187203" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.220806532Z",
					id: "176c3356-7ba2-4a1d-9ddb-c962169d7341",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.offline",
					version: "1",
				},
			],
			[
				"misspeggyx",
				{
					condition: { broadcaster_user_id: "818067359" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.221218102Z",
					id: "5e885440-c0dc-4db7-b3de-5d0036be6aa0",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.offline",
					version: "1",
				},
			],
			[
				"lcolonq",
				{
					condition: { broadcaster_user_id: "866686220" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.22148977Z",
					id: "89e28556-24bc-4197-a525-6ddaa73f8420",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.offline",
					version: "1",
				},
			],
			[
				"batatvideogames",
				{
					condition: { broadcaster_user_id: "539128874" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.222471607Z",
					id: "33814e1b-1132-47e7-b8cb-75241faa5770",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.offline",
					version: "1",
				},
			],
			[
				"bexvalentine",
				{
					condition: { broadcaster_user_id: "1013832529" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.222870616Z",
					id: "528eb296-403d-4acf-99af-0195a6724e73",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.offline",
					version: "1",
				},
			],
			[
				"bexvalentine",
				{
					condition: { broadcaster_user_id: "1013832529" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.224027668Z",
					id: "e3a476a7-837f-4919-958b-f9497943c857",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.online",
					version: "1",
				},
			],
			[
				"saltae",
				{
					condition: { broadcaster_user_id: "461736095" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.224127336Z",
					id: "2334f7d9-0872-4d5e-a966-462f25d3c427",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.online",
					version: "1",
				},
			],
			[
				"chikogaki",
				{
					condition: { broadcaster_user_id: "413015060" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.225951375Z",
					id: "4c3442d9-97b0-4ae2-88a6-6b3e0ea97f03",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.offline",
					version: "1",
				},
			],
			[
				"saltae",
				{
					condition: { broadcaster_user_id: "461736095" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.226951238Z",
					id: "2c93104a-84a3-4e8c-9c12-2b3f891b76f7",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.offline",
					version: "1",
				},
			],
			[
				"dearpekoe",
				{
					condition: { broadcaster_user_id: "960172116" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.227066919Z",
					id: "cb80f68d-5b68-412d-a678-c084a63d6933",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.offline",
					version: "1",
				},
			],
			[
				"unipiu",
				{
					condition: { broadcaster_user_id: "874233986" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.231286131Z",
					id: "52296941-da6e-4d61-90b6-4f54453e744f",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.online",
					version: "1",
				},
			],
			[
				"walfas",
				{
					condition: { broadcaster_user_id: "23075617" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.232419852Z",
					id: "a338f985-81f4-4a16-84d6-6d36cfb3446b",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.online",
					version: "1",
				},
			],
			[
				"milia",
				{
					condition: { broadcaster_user_id: "188503312" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.233443115Z",
					id: "efdaf703-3c88-4aff-a36a-20f3e491c634",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.online",
					version: "1",
				},
			],
			[
				"dearpekoe",
				{
					condition: { broadcaster_user_id: "960172116" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.233640019Z",
					id: "09127cc8-b365-4420-a91e-096f616794f7",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.online",
					version: "1",
				},
			],
			[
				"vacu0usly",
				{
					condition: { broadcaster_user_id: "54833441" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.233955265Z",
					id: "49d0fa28-01dc-4dea-8b31-1bafaec7a14e",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.online",
					version: "1",
				},
			],
			[
				"miffygeist",
				{
					condition: { broadcaster_user_id: "795478771" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.234133266Z",
					id: "1bb4949c-1894-4262-b13f-049315564b6f",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.online",
					version: "1",
				},
			],
			[
				"aaallycat",
				{
					condition: { broadcaster_user_id: "276477565" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.236592995Z",
					id: "81965bad-6a50-4212-a075-b003fe7386c6",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.online",
					version: "1",
				},
			],
			[
				"madmad01",
				{
					condition: { broadcaster_user_id: "864287979" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.238758309Z",
					id: "a4df7636-20a0-4eb1-9608-d28314568a3d",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.online",
					version: "1",
				},
			],
			[
				"parasi",
				{
					condition: { broadcaster_user_id: "834137500" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.238964008Z",
					id: "2af59b22-7704-485f-81e1-4864f92cde87",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.offline",
					version: "1",
				},
			],
			[
				"baikenvt",
				{
					condition: { broadcaster_user_id: "62127668" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.239040534Z",
					id: "5ce74226-07c5-40b2-9d58-e70f161f9143",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.offline",
					version: "1",
				},
			],
			[
				"myramors",
				{
					condition: { broadcaster_user_id: "478187203" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.240225577Z",
					id: "a1a9e0c9-cd8e-4dce-99bc-72eaf5206e5f",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.online",
					version: "1",
				},
			],
			[
				"odessavt",
				{
					condition: { broadcaster_user_id: "844897177" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.243995982Z",
					id: "5f8514b7-aac8-41c3-9e25-d7359e89ba2d",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.online",
					version: "1",
				},
			],
			[
				"souly_ch",
				{
					condition: { broadcaster_user_id: "94316536" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.244805702Z",
					id: "63f628b5-56aa-4390-836d-2bfb6c259d9a",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.online",
					version: "1",
				},
			],
			[
				"womfyy",
				{
					condition: { broadcaster_user_id: "263446776" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.247045194Z",
					id: "fe162d09-1b0c-496a-8780-ab1bfe191dff",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.online",
					version: "1",
				},
			],
			[
				"harupi",
				{
					condition: { broadcaster_user_id: "899965170" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.247046674Z",
					id: "22293a5b-1a51-4b34-acfe-6c0d59b424b1",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.online",
					version: "1",
				},
			],
			[
				"chocojax",
				{
					condition: { broadcaster_user_id: "26189911" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.248423049Z",
					id: "b33d85d9-85e1-42d6-9047-65e36e944580",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.online",
					version: "1",
				},
			],
			[
				"pachi",
				{
					condition: { broadcaster_user_id: "48807896" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.248642715Z",
					id: "7d724010-107a-464a-84c8-a81e0683378c",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.online",
					version: "1",
				},
			],
			[
				"kokopimento",
				{
					condition: { broadcaster_user_id: "24714810" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.249452844Z",
					id: "b278a64a-472f-4ffa-a001-79b32a3fd497",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.online",
					version: "1",
				},
			],
			[
				"bibibiscuitch",
				{
					condition: { broadcaster_user_id: "1335538461" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.250324201Z",
					id: "8f638f3a-9dd4-4707-9cbb-502ef4158611",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.online",
					version: "1",
				},
			],
			[
				"plss",
				{
					condition: { broadcaster_user_id: "103033809" },
					cost: 0,
					created_at: "2026-04-09T16:25:23.251505888Z",
					id: "01246cfa-ca46-45e1-a438-c6b55758198c",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.online",
					version: "1",
				},
			],
			[
				"myrmidonvt",
				{
					condition: { broadcaster_user_id: "83255335" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.251693743Z",
					id: "29748a6c-1529-4bb1-b0a7-3b96a8b1b73b",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.online",
					version: "1",
				},
			],
			[
				"kumomomomomomomo",
				{
					condition: { broadcaster_user_id: "786298312" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.252677253Z",
					id: "7a586736-04d9-486a-927b-7add1240686c",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.online",
					version: "1",
				},
			],
			[
				"snoozy",
				{
					condition: { broadcaster_user_id: "446955795" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.252774103Z",
					id: "110b2674-7a04-49f2-b411-1eaf0be581d8",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.online",
					version: "1",
				},
			],
			[
				"tini",
				{
					condition: { broadcaster_user_id: "122338258" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.255150725Z",
					id: "1e29465d-fd9a-4501-b833-2d32a3d0a986",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.online",
					version: "1",
				},
			],
			[
				"substituber",
				{
					condition: { broadcaster_user_id: "997068289" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.258903715Z",
					id: "d435cee1-d491-4675-abd6-6d84c7da4ebf",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.online",
					version: "1",
				},
			],
			[
				"krumroll",
				{
					condition: { broadcaster_user_id: "782458136" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.508336339Z",
					id: "e5a03ec3-8c4e-4679-9c77-75d7d68dbe9b",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.offline",
					version: "1",
				},
			],
			[
				"hempie",
				{
					condition: { broadcaster_user_id: "172265161" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.513985718Z",
					id: "3120c369-3ee3-4699-8a05-02ca9816ec5b",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.online",
					version: "1",
				},
			],
			[
				"walfas",
				{
					condition: { broadcaster_user_id: "23075617" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.514873262Z",
					id: "3009d1b6-7d01-4d56-a4a0-dfb10c2eac6e",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.offline",
					version: "1",
				},
			],
			[
				"kyoharuvt",
				{
					condition: { broadcaster_user_id: "741293014" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.522088126Z",
					id: "a6d7c072-18a9-400a-882d-587b05f25a01",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.online",
					version: "1",
				},
			],
			[
				"meiya",
				{
					condition: { broadcaster_user_id: "89007125" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.52520218Z",
					id: "9822c30f-6f46-4e58-b9d0-c32187fe7576",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.online",
					version: "1",
				},
			],
			[
				"netuserhael",
				{
					condition: { broadcaster_user_id: "592547707" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.525846139Z",
					id: "a131433a-bb02-4beb-8a53-b652f32d48cb",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.online",
					version: "1",
				},
			],
			[
				"nanolather",
				{
					condition: { broadcaster_user_id: "31086482" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.526918402Z",
					id: "a238a12f-fb05-4a42-8c45-0c72e70cbe8e",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.online",
					version: "1",
				},
			],
			[
				"niupao",
				{
					condition: { broadcaster_user_id: "512796146" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.528106579Z",
					id: "39536699-c07c-4a66-8afe-5bbb0e3e0afa",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.online",
					version: "1",
				},
			],
			[
				"sleepiebug",
				{
					condition: { broadcaster_user_id: "610533290" },
					cost: 1,
					created_at: "2026-04-09T16:25:23.530335609Z",
					id: "f66383e0-946b-4527-a4ba-8de97dd92d03",
					status: "enabled",
					transport: {
						callback: "https://api.piss.fan/callback",
						method: "webhook",
						secret: null,
					},
					type: "stream.online",
					version: "1",
				},
			],
		],
	};
};

// Hook retrieval handler
export const GET: RequestHandler = async ({ locals, cookies, request }) => {
	const logger = locals.logger.child({
		method: request.method,
	});
	logger.debug("starting helix hooks action");

	try {
		let token = cookies.get(ADMIN_SESSION_TOKEN);
		if (token == null) {
			logger.warn("missing token");
			return json({ status: 400, data: "invalid token" });
		}

		const res = await fetch(HOOKS_ENDPOINT, {
			method: "GET",
			headers: buildHeaders(false, token),
		});

		const body = await res.json();
		const status = res.status;
		if (!res.ok) {
			logger.warn({ status, error: body.err }, "failed to complete action");
			return json({ status, data: "action failed" });
		}

		const dummy = generateDummyHook();

		logger.debug({ status, data: body.data }, "action completed successfully");
		return json({ status, data: dummy.data });
	} catch (err) {
		logger.error({ error: err }, "unable to process hook query");
		return json({ status: 500, data: err });
	}
};

// Hook deletion handler
export const DELETE: RequestHandler = async ({ locals, cookies, request }) => {
	const logger = locals.logger.child({
		method: request.method,
	});
	logger.debug("starting helix hooks action");

	try {
		let token = cookies.get(ADMIN_SESSION_TOKEN);
		if (token == null) {
			logger.warn("missing token");
			return json({ status: 400, data: "invalid token" });
		}

		const res = await fetch(HOOKS_ENDPOINT, {
			method: "DELETE",
			headers: buildHeaders(false, token),
		});

		const status = res.status;
		const body = await res.json();

		if (!res.ok) {
			logger.warn({ status, error: body.err }, "failed to complete action");
			return json({ status: res.status, data: "action failed" });
		}

		logger.debug({ status: res.status }, "action completed successfully");
		return json({ status: res.status, data: body.data });
	} catch (err) {
		logger.error({ error: err }, "unable to process hook query");
		return json({ status: 500, data: err });
	}
};

// Hook reset handler
export const PUT: RequestHandler = async ({ locals, cookies, request }) => {
	const logger = locals.logger.child({
		method: request.method,
	});
	logger.debug("starting helix hooks action");
	try {
		const token = cookies.get(ADMIN_SESSION_TOKEN);

		if (token == null) {
			logger.warn("missing token");
			return json({ status: 400, data: "invalid token" });
		}

		const res = await fetch(HOOKS_ENDPOINT, {
			method: "PUT",
			headers: buildHeaders(false, token),
		});

		const body = await res.json();
		const status = res.status;
		if (!res.ok) {
			logger.warn({ status, error: body.err }, "failed to complete action");
			return json({ status: res.status, data: "action failed" });
		}

		logger.debug({ status, data: body.data }, "action completed successfully");
		return json({ status: body.status, data: body.data });
	} catch (err) {
		logger.error({ error: err }, "unable to process hook query");
		return json({ status: 500, data: err });
	}
};
