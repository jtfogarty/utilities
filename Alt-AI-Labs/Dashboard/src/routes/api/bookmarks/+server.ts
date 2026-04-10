import { error, json } from '@sveltejs/kit';
import { fetchBookmarks } from '$lib/server/bookmarksRepo';
import type { RequestHandler } from './$types';

export const GET: RequestHandler = async ({ url }) => {
	const limitRaw = Number(url.searchParams.get('limit') ?? 500);
	const offsetRaw = Number(url.searchParams.get('offset') ?? 0);
	const limit = Math.min(2000, Math.max(1, Number.isFinite(limitRaw) ? limitRaw : 500));
	const offset = Math.max(0, Number.isFinite(offsetRaw) ? offsetRaw : 0);

	try {
		const rows = await fetchBookmarks(limit, offset);
		return json(rows);
	} catch (e) {
		const message = e instanceof Error ? e.message : 'SurrealDB error';
		console.error('[api/bookmarks]', message);
		error(502, message);
	}
};
