<script lang="ts">
	import { onMount } from 'svelte';
	import { Button } from '$lib/components/ui/button/index.js';
	import { Skeleton } from '$lib/components/ui/skeleton/index.js';
	import BookmarkGrid from '$lib/components/bookmarks/BookmarkGrid.svelte';
	import BookmarkList from '$lib/components/bookmarks/BookmarkList.svelte';
	import BookmarksEmpty from '$lib/components/bookmarks/BookmarksEmpty.svelte';
	import {
		allBookmarks,
		bookmarksError,
		bookmarksLoading,
		filteredBookmarks,
		loadBookmarks,
	} from '$lib/stores/bookmarks';

	type ViewMode = 'list' | 'card';
	let mode = $state<ViewMode>('card');

	onMount(() => {
		void loadBookmarks();
	});
</script>

<svelte:head>
	<title>Browse — Alt-AI-Labs Bookmarks</title>
	<meta name="description" content="Browse and search X bookmarks from SurrealDB." />
</svelte:head>

<div class="space-y-6 p-4 lg:p-6">
	<div class="flex flex-col gap-4 sm:flex-row sm:items-center sm:justify-between">
		<div>
			<h1 class="text-foreground text-2xl font-semibold tracking-tight">Browse</h1>
			<p class="text-muted-foreground mt-1 text-sm">
				{$filteredBookmarks.length} shown
				{#if $allBookmarks.length !== $filteredBookmarks.length}
					of {$allBookmarks.length} loaded
				{/if}
			</p>
		</div>
		<div class="flex gap-2">
			<Button
				type="button"
				variant={mode === 'list' ? 'default' : 'outline'}
				size="sm"
				onclick={() => (mode = 'list')}
			>
				List
			</Button>
			<Button
				type="button"
				variant={mode === 'card' ? 'default' : 'outline'}
				size="sm"
				onclick={() => (mode = 'card')}
			>
				Cards
			</Button>
		</div>
	</div>

	{#if $bookmarksError}
		<div
			class="border-destructive/40 bg-destructive/10 text-destructive rounded-lg border px-4 py-3 text-sm"
			role="alert"
		>
			<strong class="font-medium">Database error.</strong>
			{$bookmarksError}
		</div>
	{/if}

	{#if $bookmarksLoading}
		<div class="space-y-3">
			<Skeleton class="h-10 w-full max-w-md rounded-md" />
			<Skeleton class="h-40 w-full rounded-xl" />
			<Skeleton class="h-40 w-full rounded-xl" />
		</div>
	{:else if !$filteredBookmarks.length && !$allBookmarks.length}
		<BookmarksEmpty message="No bookmarks loaded. Start SurrealDB and ensure x_bookmarks has data." />
	{:else if !$filteredBookmarks.length}
		<BookmarksEmpty message="No bookmarks match your search." />
	{:else if mode === 'list'}
		<BookmarkList bookmarks={$filteredBookmarks} />
	{:else}
		<BookmarkGrid bookmarks={$filteredBookmarks} />
	{/if}
</div>
