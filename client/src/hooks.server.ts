import { redirect, type Handle } from '@sveltejs/kit';
import { CHANNELS, ROOT_HOSTNAME, API_HOSTNAME, ROOT_SUBDOMAIN } from '$lib/client/constants';

export const handle: Handle = async ({ event, resolve }) => {
	const host = event.request.headers.get('host');
	const url = new URL(event.request.url);

	const subdomain = host?.split('.')[0];

	if (
		!host ||
		host === ROOT_HOSTNAME ||
		host === API_HOSTNAME ||
		!subdomain ||
		subdomain === ROOT_SUBDOMAIN
	) {
		return resolve(event);
	}

	const shouldReroute = isRoutable(subdomain);
	if (shouldReroute) {
		event.locals.channel = subdomain;
		return resolve(event);
	} else {
		const redirectUrl = `https://${ROOT_HOSTNAME}${event.url.pathname}${event.url.search}`;
		redirect(302, redirectUrl);
	}
};

const isRoutable = (subdomain: string) => {
	return CHANNELS.includes(subdomain);
};
