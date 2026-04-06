# Alt-AI-Labs Bookmark Dashboard — Architecture & Technology Guide

> **Goal**: Build a modern, AI-augmented bookmark dashboard that reads data from SurrealDB (`bookmarks` namespace / `v1` database / `x_bookmarks` table) and lets users explore, browse, and query their X bookmarks with tree views, card layouts, list views, and an embedded AI agent.

---

## 1. Framework Decision: Svelte vs Vue

Both are excellent choices. Here is an honest side-by-side for *this specific project*.

| Criterion | **Svelte / SvelteKit** | **Vue 3 / Nuxt** |
|---|---|---|
| **Bundle size** | ✅ Smallest — compiles away the framework | ❌ Slightly heavier runtime |
| **Reactivity model** | ✅ Compiler-driven, zero boilerplate | 🟡 Composition API is great but more explicit |
| **shadcn support** | ✅ `shadcn-svelte` — actively maintained, near feature-parity | ✅ `shadcn-vue` — also actively maintained, slightly older |
| **Dashboard templates** | 🟡 Good but smaller ecosystem | ✅ Larger template ecosystem (e.g. Vuestic, Vue-Element-Plus-Admin) |
| **TypeScript DX** | ✅ First-class with SvelteKit | ✅ First-class with Vue 3 |
| **Learning curve** | ✅ Lower — feels like enhanced HTML | 🟡 Options API is approachable; Composition API takes adjustment |
| **Community / ecosystem** | 🟡 Rapidly growing, smaller than Vue | ✅ Mature, large ecosystem |
| **AI/LLM component libs** | 🟡 Growing | 🟡 Growing |
| **SurrealDB JS SDK** | ✅ Works natively in either (same `surrealdb.js` SDK) | ✅ Same |

### Recommendation: **SvelteKit**

For a project that is:
- Primarily **data-display** focused (bookmarks, cards, trees)
- Backed by a developer who already works in Rust (Svelte's compiler philosophy resonates)
- Wanting the **smallest possible shipping bundle**
- Using **shadcn-svelte** which is the most actively developed shadcn port

SvelteKit is the stronger choice. However, if you are already very comfortable with Vue 3 / the Options API, Vue is completely viable and you will not be blocked.

> **Bottom line**: Use **SvelteKit** with `shadcn-svelte`. If you change your mind later, the component API surface is similar enough that migration is manageable.

---

## 2. Full Technology Stack

### Core Framework
- **[SvelteKit](https://kit.svelte.dev/)** — Full-stack framework (SSR + SPA modes, file-based routing, form actions)
- **[TypeScript](https://www.typescriptlang.org/)** — Strict typing throughout

### UI Component Library
- **[shadcn-svelte](https://www.shadcn-svelte.com/)** — Copy-paste component library built on top of bits-ui and Tailwind. Components you own, not a dependency to manage.
  - Key components in use: `Sidebar`, `Card`, `Button`, `Input`, `Badge`, `ScrollArea`, `Separator`, `Tooltip`, `Sheet`, `Dialog`, `Combobox`

### Styling
- **[Tailwind CSS v4](https://tailwindcss.com/)** — Required by shadcn-svelte. Use CSS variables for theming.
- **Custom CSS variables** for token overrides (dark mode, brand colors)

### Dashboard Template (starting point)
Consider one of these as a scaffold:
1. **[shadcn-svelte admin dashboard example](https://github.com/shadcn-svelte/examples)** — Minimal, matches shadcn-svelte idioms exactly. **Recommended.**
2. **[Flowbite Svelte Admin Dashboard](https://github.com/themesberg/flowbite-svelte-admin-dashboard)** — More feature-rich out of the box.
3. **[SvelteKit Dashboard Starter](https://github.com/Huxwell/sveltekit-dashboard)** — Lightweight community template.

> **DECISION** Lets use this one.  exactly like https://flowbite-svelte.com/admin-dashboard

### Database / Data Layer
- **[SurrealDB JS SDK](https://surrealdb.com/docs/sdk/javascript)** (`surrealdb`) — Connect from the browser or server
  - Namespace: `bookmarks`
  - Database: `v1`
  - Primary table: `x_bookmarks` (key field: `raw_json`)
- Connection via **WebSocket** (`ws://127.0.0.1:8000`) for live queries (SurrealDB Live queries enable real-time bookmark sync)

### State Management
- **Svelte stores** (`writable`, `derived`) — Sufficient for most UI state
- **[nanostores](https://github.com/nanostores/nanostores)** (optional) — If cross-component state grows complex

### AI Agent Integration
- **[Ollama](https://ollama.com/)** — Local LLM runtime (no API keys, fully private, runs on-device)
  - Recommended models: `llama3`, `mistral`, or `phi3` depending on hardware
- **[ollama-js](https://github.com/ollama/ollama-js)** — Official Ollama JS/TS client for streaming chat completions
- **[Vercel AI SDK](https://sdk.vercel.ai/)** (`ai` package, optional) — Use only if you later want to swap in a cloud model; Ollama provider is available via `@ai-sdk/ollama`
- The AI agent receives bookmark context (text excerpts from `raw_json`) via system prompt; no RAG infrastructure needed for MVP

### Search
- **Client-side**: [Fuse.js](https://fusejs.io/) — Fuzzy search over in-memory bookmark records
- **Server-side / AI search**: SurrealDB full-text search (`SEARCH ANALYZER`) or vector embeddings (future)

### Icons
- **[Lucide Svelte](https://lucide.dev/guide/packages/lucide-svelte)** — Matches shadcn design language

### Tooling
- **[Vite](https://vitejs.dev/)** — Already baked into SvelteKit
- **[Vitest](https://vitest.dev/)** — Unit testing
- **[Playwright](https://playwright.dev/)** — E2E testing (optional)
- **ESLint + Prettier** with `eslint-plugin-svelte`

---

## 3. Project Structure

```
dashboard/
├── src/
│   ├── lib/
│   │   ├── components/
│   │   │   ├── ui/              # shadcn-svelte components (auto-generated)
│   │   │   ├── layout/
│   │   │   │   ├── AppShell.svelte
│   │   │   │   ├── Sidebar.svelte
│   │   │   │   └── Header.svelte
│   │   │   ├── bookmarks/
│   │   │   │   ├── BookmarkCard.svelte    # X-style card view
│   │   │   │   ├── BookmarkList.svelte    # List view
│   │   │   │   └── BookmarkGrid.svelte    # Grid wrapper
│   │   │   └── ai/
│   │   │       ├── ChatPanel.svelte
│   │   │       └── MessageBubble.svelte
│   │   ├── stores/
│   │   │   ├── bookmarks.ts     # Svelte store + SurrealDB queries
│   │   │   ├── ui.ts            # Sidebar open/closed, active view, etc.
│   │   │   └── ai.ts            # Chat history, streaming state
│   │   ├── db/
│   │   │   ├── client.ts        # SurrealDB connection singleton
│   │   │   └── queries.ts       # SurrealQL query helpers
│   │   └── types/
│   │       └── bookmark.ts      # TypeScript types derived from raw_json schema
│   ├── routes/
│   │   ├── layout.css           # Tailwind + shadcn CSS variables (auto-generated)
│   │   ├── +layout.svelte       # AppShell wrapper
│   │   ├── +page.svelte         # Default redirect → /browse
│   │   ├── browse/
│   │   │   └── +page.svelte     # All bookmarks, list/card toggle
│   │   └── ai/
│   │       └── +page.svelte     # AI chat panel
│   └── app.html
├── static/
├── package.json
├── svelte.config.js
├── tailwind.config.js
└── vite.config.ts
```

---

## 4. Key UI Sections

### 4.1 Header / Toolbar
- App logo / title: `Alt-AI-Labs Bookmarks`
- Global search bar (Fuse.js powered, opens command palette-style with `shadcn Command`)
- User avatar / settings icon (right side)

### 4.2 Sidebar
Icon-first navigation with collapsible labels:

| Icon | Route | Description |
|---|---|---|
| `BookOpen` | `/browse` | All bookmarks |
| `Bot` | `/ai` | AI agent |

> **Note**: Tree/folder view is **out of scope for v1** — `raw_json` does not contain folder metadata.

Use `shadcn-svelte`'s `Sidebar` + `SidebarNav` patterns.

### ~~4.3 Tree View~~ — Skipped for v1
Folders do not exist in `raw_json`. This view is deferred until a tagging/categorization feature is built.

### 4.3 Browse View (`/browse`)
- All bookmarks, paginated or virtualized (use `svelte-virtual-list` for large datasets)
- Toggle: **List** ↔ **Card**
- Filter chips: by date, author, hashtag

### 4.4 AI View (`/ai`)
- Streaming chat interface powered by **local Ollama**
- Agent reads bookmarks from the store as context (injected into system prompt)
- Suggested prompts: "Summarize my top saved threads", "Find bookmarks about Rust", "What topics do I bookmark most?"
- No external API calls — fully private, works offline

---

## 5. Data Layer Design

```typescript
// src/lib/types/bookmark.ts

export interface XBookmark {
  id: string;
  raw_json: {
    id: string;
    text: string;
    author_id: string;
    created_at: string;
    entities?: {
      urls?: { expanded_url: string; display_url: string }[];
      hashtags?: { tag: string }[];
    };
    // ... other X API tweet fields
  };
  created_at?: string;
  folder?: string;
}
```

SurrealQL query pattern:
```sql
SELECT * FROM x_bookmarks LIMIT 50;
SELECT * FROM x_bookmarks WHERE raw_json.text CONTAINS $query;
```

---

## 6. Bootstrap Commands

```bash
# 1. Create SvelteKit project (minimal template, TypeScript, no add-ons)
npx sv create . --template minimal --types ts --no-add-ons

# 2. Add Tailwind CSS (required by shadcn-svelte)
npx sv add tailwindcss

# 3. Initialize shadcn-svelte (choose "dark" theme when prompted)
npx shadcn-svelte@latest init

# 4. Add core shadcn-svelte components
npx shadcn-svelte@latest add sidebar card button input badge scroll-area separator tooltip sheet dialog command

# 5. Add SurrealDB JS SDK
bun add surrealdb

# 6. Add Lucide icons
bun add lucide-svelte

# 7. Add Fuse.js for client-side fuzzy search
bun add fuse.js

# 8. Add Ollama JS client (local AI — no API key required)
bun add ollama
# Optional: Vercel AI SDK with Ollama provider if you want streaming helpers
# bun add ai @ai-sdk/ollama

# 9. Ensure Ollama is running locally with your chosen model
# ollama pull llama3        # or mistral, phi3, etc.
# ollama serve              # starts on http://localhost:11434

# 10. Start the dev server
bun run dev
```

> **Prerequisite**: [Install Ollama](https://ollama.com/download) on your Mac before running step 9.

---

## 7. Decisions (Resolved)

| Decision | Choice | Notes |
|---|---|---|
| **Framework** | SvelteKit | File-based routing, SPA mode, TypeScript first |
| **SurrealDB connection** | SvelteKit API route (proxy) | Proxies WebSocket on server side; safer than direct browser connection |
| **AI model** | Local Ollama | Fully private, no API costs, runs on-device at `http://localhost:11434` |
| **Folder / tree view** | ⛔ Skipped for v1 | Folders don't exist in `raw_json`; defer until tagging feature is built |
| **Authentication** | Single user | No login system needed; app is local-only |
| **UI theme** | Dark mode (default) | Set at `shadcn-svelte init` time; no toggle needed for v1 |

---

## 8. Vue Alternative (if you change your mind)

If you prefer Vue 3, the stack would be:

| Role | Tool |
|---|---|
| Framework | Vue 3 + Vite (or Nuxt 3 for SSR) |
| Components | `shadcn-vue` |
| State | Pinia |
| Routing | Vue Router |
| Template | [vue-pure-admin](https://github.com/pure-admin/vue-pure-admin) or [Vuestic Admin](https://github.com/epicmaxco/vuestic-admin) |
| DB | Same `surrealdb` JS SDK |
| AI | Same Vercel AI SDK |

The component structure and feature set would be identical — only the syntax and reactivity model differ.
