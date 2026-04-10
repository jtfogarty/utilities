<script lang="ts">
	import './layout.css';
	import favicon from '$lib/assets/favicon.svg';
	import Header from '$lib/components/layout/Header.svelte';
	import AppSidebar from '$lib/components/layout/AppSidebar.svelte';

	let { children } = $props();
	let drawerOpen = $state(false);
</script>

<svelte:head><link rel="icon" href={favicon} /></svelte:head>

<!-- Shell uses shadcn semantic colors so header / sidebar / main stay one cohesive dark (or light) theme -->
<header
	class="border-border bg-card fixed top-0 z-40 mx-auto w-full flex-none border-b shadow-sm dark:border-zinc-600/80"
>
	<Header bind:drawerOpen />
</header>

<div class="overflow-hidden lg:flex">
	<AppSidebar bind:drawerOpen />
	<main
		class="border-border bg-background relative min-h-[calc(100dvh-var(--app-header-height))] w-full overflow-y-auto pt-[var(--app-header-height)] lg:ml-64 lg:border-l lg:border-border"
	>
		{@render children()}
	</main>
</div>
