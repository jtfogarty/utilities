# Dashboard — Next Steps Build Plan

## Current State

| Layer | Status |
|---|---|
| SvelteKit scaffold | ✅ Done |
| Tailwind CSS | ✅ Done |
| shadcn-svelte (Vega / Inter / Dark) | ✅ Done |
| shadcn components installed | ✅ Done (`sidebar`, `card`, `button`, `input`, `badge`, etc.) |
| `surrealdb`, `lucide-svelte`, `fuse.js`, `ollama` | ✅ Done |
| `.env` — namespace fixed to `bookmarks` | ✅ Done |
| Flowbite reference cloned | ✅ `reference/flowbite-admin/` (gitignored) |
| App shell / layout | ❌ Default SvelteKit placeholder |
| Routes (`/browse`, `/ai`) | ❌ Not created |
| SurrealDB connection | ❌ Not wired |
| Bookmark display (list / card) | ❌ Not built |
| AI chat panel | ❌ Not built |

### ⚠️ Component Library Note
The Flowbite admin dashboard (`reference/flowbite-admin/`) uses **Flowbite Svelte** components (`flowbite-svelte` / `flowbite-svelte-icons`). These **cannot be copy-pasted** into our project — we use **shadcn-svelte**.

**Strategy**: The Flowbite source reveals the *layout structure and Tailwind class patterns* we need. We reimplement the same shell using shadcn-svelte components.

Key reference files (read-only, do not copy directly):
- `reference/flowbite-admin/src/routes/(sidebar)/+layout.svelte` — fixed header + flex container + 64px sidebar + offset content
- `reference/flowbite-admin/src/routes/(sidebar)/Navbar.svelte` — search bar center, icons right
- `reference/flowbite-admin/src/routes/(sidebar)/Sidebar.svelte` — data-array driven nav items

---

## Phase 1 — App Shell (do this first)

> Get the permanent chrome (header + sidebar + content area) in place. Layout pattern derived from `reference/flowbite-admin/src/routes/(sidebar)/+layout.svelte`.

### 1.1 Update `app.html` — activate dark mode
Add `class="dark"` to `<html>` so shadcn-svelte's dark CSS variables activate permanently:
```html
<html lang="en" class="dark">
```

### 1.2 Update `src/routes/+layout.svelte` — build the shell
Replace the default layout with the Flowbite-derived shell structure:
```svelte
<script lang="ts">
  import './layout.css';
  import Header from '$lib/components/layout/Header.svelte';
  import AppSidebar from '$lib/components/layout/AppSidebar.svelte';
  let { children } = $props();
</script>

<!-- Fixed header, full width, z-40 -->
<header class="fixed top-0 z-40 w-full border-b border-border bg-background">
  <Header />
</header>

<!-- Sidebar + content wrapper -->
<div class="overflow-hidden lg:flex">
  <AppSidebar />
  <!-- Content offset: 70px top (header) + 256px left (sidebar on lg) -->
  <main class="relative h-full w-full overflow-y-auto pt-[70px] lg:ml-64">
    {@render children()}
  </main>
</div>
```

### 1.3 Create `src/lib/components/layout/Header.svelte`
Based on Flowbite's `Navbar.svelte` pattern, built with shadcn:
- Left: App logo + title (`Alt-AI-Labs Bookmarks`)
- Center: shadcn `<Input>` search bar (full width on md+)
- Right: search icon button

### 1.4 Create `src/lib/components/layout/AppSidebar.svelte`
Based on Flowbite's `Sidebar.svelte` data-driven pattern:
- Fixed left, `w-64`, `h-screen`, `mt-[70px]`
- Nav items as a data array → rendered in `{#each}`
- Two items: Browse (`BookOpen` icon, `/browse`) and AI (`Bot` icon, `/ai`)
- Use shadcn `<Button variant="ghost">` for nav items

---

## Phase 2 — SurrealDB Connection

> Required before any real data can appear.

### 2.1 Create `src/lib/db/client.ts`
```typescript
import Surreal from 'surrealdb';

let db: Surreal | null = null;

export async function getDb(): Promise<Surreal> {
  if (db) return db;
  db = new Surreal();
  await db.connect('ws://127.0.0.1:8000');
  await db.use({ namespace: 'bookmarks', database: 'v1' });
  return db;
}
```

### 2.2 Create `src/lib/db/queries.ts`
- `fetchBookmarks(limit, offset)` — paginated SELECT
- `searchBookmarks(query)` — WHERE raw_json.text CONTAINS
- `fetchBookmarkById(id)`

### 2.3 Create `src/lib/types/bookmark.ts`
TypeScript interfaces matching the `raw_json` shape from the X API.  
Run a quick SurrealDB query first to see the actual field names:
```sql
SELECT raw_json FROM x_bookmarks LIMIT 1;
```

### 2.4 Create `src/lib/stores/bookmarks.ts`
Svelte writable store that calls `fetchBookmarks()` on load and caches results in memory for Fuse.js.

---

## Phase 3 — Browse View (`/browse`)

> The core data display page.

### 3.1 Create `src/routes/browse/+page.svelte`
- On mount: load bookmarks from store
- Render toggle button: **List** ↔ **Card**
- Pass bookmarks to the active view component

### 3.2 Create `src/lib/components/bookmarks/BookmarkCard.svelte`
X.com-style card using shadcn `Card`:
- Author name + handle + avatar placeholder
- Tweet text (from `raw_json.text`)
- Timestamp
- Link chip(s) (from `raw_json.entities.urls`)
- Hashtag badges (from `raw_json.entities.hashtags`)

### 3.3 Create `src/lib/components/bookmarks/BookmarkList.svelte`
Compact single-line row per bookmark:
- Author handle | truncated text | date

### 3.4 Create `src/lib/components/bookmarks/BookmarkGrid.svelte`
Responsive grid wrapper that receives bookmarks and renders `BookmarkCard` in a CSS grid.

### 3.5 Wire Fuse.js search to the Header search bar
- On input, filter the in-memory bookmarks store
- Debounce 300ms

---

## Phase 4 — AI Chat View (`/ai`)

> Local Ollama-powered chat that can answer questions about bookmarks.

### 4.1 Create `src/routes/ai/+page.svelte`
- Chat message history (Svelte store)
- Text input + send button
- Render `ChatPanel.svelte`

### 4.2 Create `src/lib/components/ai/ChatPanel.svelte`
- Scrollable message list
- Auto-scroll to latest message
- Loading/streaming indicator

### 4.3 Create `src/lib/components/ai/MessageBubble.svelte`
- User vs assistant styling
- Markdown rendering (optional: `marked` or `svelte-markdown`)

### 4.4 Create `src/lib/stores/ai.ts`
- `messages` writable store (array of `{role, content}`)
- `isStreaming` boolean

### 4.5 Create `src/routes/api/chat/+server.ts`
SvelteKit server route that proxies to Ollama:
```typescript
import { json } from '@sveltejs/kit';
import { Ollama } from 'ollama';

const ollama = new Ollama({ host: 'http://localhost:11434' });

export async function POST({ request }) {
  const { messages, bookmarkContext } = await request.json();
  const stream = await ollama.chat({
    model: 'llama3',
    messages: [
      { role: 'system', content: `You are a bookmark assistant. Here are the user's bookmarks:\n${bookmarkContext}` },
      ...messages
    ],
    stream: true,
  });
  // Return as streaming response
}
```

---

## Phase 5 — Polish

- [ ] Redirect `/` → `/browse` in `+page.svelte`
- [ ] Loading skeletons while bookmarks fetch (use shadcn `Skeleton`)
- [ ] Empty state component (no results found)
- [ ] Pagination or virtual scroll for large bookmark lists (`svelte-virtual-list`)
- [ ] Error boundary/toast for SurrealDB connection failures (shadcn `Toast`)
- [ ] Page titles / meta tags per route

---

## Recommended Build Order

```
Phase 1 (Shell) → Phase 2 (DB) → Phase 3 (Browse) → Phase 4 (AI) → Phase 5 (Polish)
```

Start with **Phase 1** — until the shell is in place, you can't visually verify anything else.
