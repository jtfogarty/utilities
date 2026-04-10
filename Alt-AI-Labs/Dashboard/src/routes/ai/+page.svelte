<script lang="ts">
	import { onMount } from 'svelte';
	import ChatPanel from '$lib/components/ai/ChatPanel.svelte';
	import { loadBookmarks } from '$lib/stores/bookmarks';

	onMount(() => {
		void loadBookmarks();
	});
</script>

<svelte:head>
	<title>AI — Alt-AI-Labs Bookmarks</title>
	<meta name="description" content="Ask questions about your bookmarks with local Ollama." />
</svelte:head>

<div class="mx-auto max-w-3xl space-y-6 p-4 pb-10 lg:p-8">
	<div class="space-y-2">
		<h1 class="text-foreground text-2xl font-semibold tracking-tight md:text-3xl">AI assistant</h1>
		<p class="text-muted-foreground max-w-2xl text-sm leading-relaxed md:text-[15px]">
			Runs against local Ollama (<code
				class="bg-muted text-foreground rounded-md px-1.5 py-0.5 font-mono text-xs">llama3</code
			>
			by default). Start
			<code class="bg-muted text-foreground rounded-md px-1.5 py-0.5 font-mono text-xs"
				>ollama serve</code
			>
			and pull a model first. Bookmark text is included as context on each message.
		</p>
	</div>
	<ChatPanel />
</div>
