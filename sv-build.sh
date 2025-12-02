#!/bin/bash

# Check if a project name is provided
if [ -z "$1" ]; then
  echo "Usage: $0 <project-name>"
  exit 1
fi

PROJECT_NAME="$1"

# Create the SvelteKit project non-interactively (Skeleton template, TypeScript, ESLint, Prettier)
bun x sv create "$PROJECT_NAME" --template minimal --types ts --add eslint prettier --install bun

# Navigate to the project directory
cd "$PROJECT_NAME"

# Set up Tailwind CSS integration (installs package, creates configs, and global CSS file; no plugins)
bun x sv add tailwindcss="plugins:none" --install bun

# Run shadcn-svelte init with your exact config (overwrites Tailwind CSS file as needed)
bun x shadcn-svelte@latest init --base-color slate --css src/routes/layout.css --lib-alias '$lib' --components-alias '$lib/components' --utils-alias '$lib/utils' --hooks-alias '$lib/hooks' --ui-alias '$lib/components/ui'

# Install mode-watcher (for dark/light mode handling)
bun add mode-watcher

# Add the specified shadcn-svelte components non-interactively
bun x shadcn-svelte@latest add -y button button-group checkbox alert card command chart
