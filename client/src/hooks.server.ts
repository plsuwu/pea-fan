import { redirect, type Handle } from '@sveltejs/kit';
import { PROTO, PF_BASE_URL, PF_API_URL, channels } from '$lib/utils';

export const handle: Handle = async ({ event, resolve }) => {
	// const url = new URL(event.request.url);
	//
	const host = event.request.headers.get('host');
	const subdomain = host?.split('.')[0];

	if (!host || host === PF_BASE_URL || host === PF_API_URL || !subdomain) {
		return resolve(event);
	}

	const shouldReroute = isRoutable(subdomain);
	if (shouldReroute) {
		event.locals.currentChannel = subdomain;
		return resolve(event);
	} else {
		const redirectUrl = `${PROTO}://${PF_BASE_URL}${event.url.pathname}${event.url.search}`;
		redirect(302, redirectUrl);
	}
};

const isRoutable = (subdomain: string) => {
	return channels.includes(subdomain);
};
