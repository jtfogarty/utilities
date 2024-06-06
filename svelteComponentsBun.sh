#!/bin/bash

# Install Tailwind CSS
bun add -d tailwindcss postcss autoprefixer
npx tailwindcss init -p

# Create Tailwind CSS configuration file
cat > tailwind.config.js <<EOL
module.exports = {
  content: [
    "./src/**/*.{html,js,svelte,ts}",
    "./public/index.html",
  ],
  theme: {
    extend: {},
  },
  plugins: [],
}
EOL

# Create Tailwind CSS main stylesheet
mkdir -p src/lib
cat > src/lib/tailwind.css <<EOL
@tailwind base;
@tailwind components;
@tailwind utilities;
EOL

# Import Tailwind CSS in the main Svelte file
sed -i "s|</style>|@import 'tailwindcss/tailwind.css';\n</style>|" src/app.html

# Install shadcn-svelte
bun add shadcn-svelte

# Create a directory for shadcn components if it doesn't exist
mkdir -p src/lib/components

# Read components from shadcn.txt and install them
while IFS= read -r component; do
  bun run shadcn-svelte add $component
done < shadcn.txt

echo "Tailwind CSS and shadcn-svelte components installation is complete."
