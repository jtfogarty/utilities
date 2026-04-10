import { error, json } from '@sveltejs/kit';
import { inspectLinkForPreview } from '$lib/server/linkPreview';
import type { RequestHandler } from './$types';

export const GET: RequestHandler = async ({ url, request }) => {
	const raw = url.searchParams.get('url')?.trim();
	if (!raw) error(400, 'Missing url');

	const origin = request.headers.get('origin');

	try {
		const payload = await inspectLinkForPreview(raw, origin);
		return json(payload, {
			headers: { 'Cache-Control': 'private, max-age=300' },
		});
	} catch (e) {
		const message = e instanceof Error ? e.message : 'Link preview failed';
		error(400, message);
	}
};
