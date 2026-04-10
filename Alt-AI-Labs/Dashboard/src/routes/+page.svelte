<script lang="ts">
	import { goto } from '$app/navigation';
	import { onMount } from 'svelte';

	onMount(() => {
		void goto('/browse', { replaceState: true });
	});
</script>

<svelte:head>
	<title>Alt-AI-Labs Bookmarks</title>
</svelte:head>

<p class="text-muted-foreground p-6 text-sm">Redirecting to browse…</p>
