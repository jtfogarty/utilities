#!/bin/bash

# Check if all required arguments are provided
if [ "$#" -lt 3 ]; then
    echo "Usage: $0 <package_manager> <project_name> <components> [brand]"
    echo "Example: $0 bun my_project 'button,card,input' mybrand"
    exit 1
fi

manager=$1
project=$2
components=$3
brand=$4

# Initialize Wails project
wails init -n $project -t svelte
cd $project

# Update wails.json
sed -i '' "s|npm|$manager|g" wails.json
sed -i '' 's|"auto",|"auto",\n  "wailsjsdir": "./frontend/src/lib",|' wails.json

# Update main.go
sed -i '' "s|all:frontend/dist|all:frontend/build|" main.go

# Handle branding if specified
if [[ -n $brand ]]; then
    mv frontend/src/App.svelte +page.svelte
    sed -i '' "s|'./assets|'\$lib/assets|" +page.svelte
    sed -i '' "s|'../wails|'\$lib/wails|" +page.svelte
    mv frontend/src/assets .
fi

# Remove old frontend and create new Svelte app
rm -r frontend
$manager x create-svelte@latest frontend

# Move files for branding
if [[ -n $brand ]]; then
    mv +page.svelte frontend/src/routes/+page.svelte
    mkdir -p frontend/src/lib
    mv assets frontend/src/lib/
fi

# Install dependencies and configure Svelte
cd frontend
$manager install
$manager uninstall @sveltejs/adapter-auto
$manager add -D @sveltejs/adapter-static

# Add TailwindCSS
$manager x svelte-add@latest tailwindcss
$manager install

# Setup shadcn-svelte
$manager add -D shadcn-svelte
$manager x shadcn-svelte@latest init

# Add specified shadcn components
IFS=',' read -ra ADDR <<< "$components"
for component in "${ADDR[@]}"; do
    $manager x shadcn-svelte@latest add $component
done

# Create +layout.ts
echo "export const prerender = true;" > src/routes/+layout.ts
echo "export const ssr = false;" >> src/routes/+layout.ts

# Update svelte.config.js
cat > svelte.config.js << EOL
import adapter from '@sveltejs/adapter-static';
import { vitePreprocess } from '@sveltejs/vite-plugin-svelte';

/** @type {import('@sveltejs/kit').Config} */
const config = {
    preprocess: vitePreprocess(),
    kit: {
        adapter: adapter({
            pages: 'build',
            assets: 'build',
            fallback: null,
            precompress: false,
            strict: true
        }),
        alias: {
            "\$lib": "./src/lib",
            "@/*": "./src/lib/*"
        }
    }
};

export default config;
EOL

# Return to project root
cd ..

echo "Setup complete. I will now run 'wails dev' to start the development server."

# Start Wails development server
wails dev
