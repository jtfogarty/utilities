import { error } from '@sveltejs/kit';
import { env } from '$env/dynamic/private';
import { Ollama } from 'ollama';
import type { RequestHandler } from './$types';

export const POST: RequestHandler = async ({ request }) => {
	let body: {
		messages?: { role: string; content: string }[];
		bookmarkContext?: string;
		model?: string;
	};
	try {
		body = (await request.json()) as typeof body;
	} catch {
		error(400, 'Invalid JSON body');
	}

	const msgs = body.messages ?? [];
	const bookmarkContext = body.bookmarkContext ?? '(no bookmarks loaded)';
	const model = body.model ?? 'llama3';
	const host = env.OLLAMA_HOST ?? 'http://127.0.0.1:11434';

	const ollama = new Ollama({ host });

	try {
		const stream = await ollama.chat({
			model,
			messages: [
				{
					role: 'system',
					content: `You are a bookmark assistant. The user has saved X/Twitter bookmarks. Use the following excerpts as context when answering:\n\n${bookmarkContext}`,
				},
				...msgs.filter((m) => m.role === 'user' || m.role === 'assistant'),
			],
			stream: true,
		});

		const encoder = new TextEncoder();
		const readable = new ReadableStream({
			async start(controller) {
				try {
					for await (const part of stream) {
						controller.enqueue(encoder.encode(`${JSON.stringify(part)}\n`));
					}
				} catch (e) {
					const msg = e instanceof Error ? e.message : 'stream error';
					controller.enqueue(encoder.encode(`${JSON.stringify({ error: msg })}\n`));
				} finally {
					controller.close();
				}
			},
		});

		return new Response(readable, {
			headers: {
				'Content-Type': 'application/x-ndjson; charset=utf-8',
				'Cache-Control': 'no-cache',
			},
		});
	} catch (e) {
		error(500, e instanceof Error ? e.message : 'Ollama error');
	}
};
