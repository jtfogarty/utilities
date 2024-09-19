# Utilities to help automate development tasks


## Wails-Svelte-Shadcn Project Setup Script Explanation (create.sh)

This markdown file explains a bash script that sets up a project using Wails, Svelte, and shadcn-svelte. The script automates the process of creating a new project with these technologies and configuring it with specified components.

### Script Overview

The script performs the following main tasks:
1. Initializes a Wails project with Svelte
2. Sets up a new Svelte app
3. Configures TailwindCSS
4. Installs and configures shadcn-svelte
5. Sets up static adapter for Svelte
6. Configures the project for development

### Usage

```bash
./script_name.sh <package_manager> <project_name> <components> [brand]
```

- `<package_manager>`: The package manager to use (e.g., npm, yarn, bun)
- `<project_name>`: The name of your project
- `<components>`: A comma-separated list of shadcn components to add
- `[brand]`: Optional. If specified, sets up additional branding configurations

Example:
```bash
./script_name.sh bun my_project 'button,card,input' mybrand
```

### Detailed Explanation

1. **Argument Checking**
   - The script checks if at least 3 arguments are provided.
   - If not, it displays usage instructions and exits.

2. **Wails Project Initialization**
   - Initializes a new Wails project with Svelte template.
   - Updates `wails.json` to use the specified package manager and set the Wails JS directory.
   - Modifies `main.go` to point to the correct build directory.

3. **Branding Setup (Optional)**
   - If a brand is specified, it moves and renames some files for branding purposes.

4. **Svelte App Setup**
   - Removes the old frontend and creates a new Svelte app.
   - Moves branding files if applicable.

5. **Dependency Installation and Configuration**
   - Installs dependencies in the frontend directory.
   - Replaces `@sveltejs/adapter-auto` with `@sveltejs/adapter-static`.

6. **TailwindCSS Setup**
   - Adds TailwindCSS to the project using `svelte-add`.

7. **shadcn-svelte Setup**
   - Installs and initializes shadcn-svelte.
   - Adds specified shadcn components.

8. **Svelte Configuration**
   - Creates a `+layout.ts` file to enable prerendering and disable SSR.
   - Updates `svelte.config.js` to use the static adapter and set up aliases.

9. **Development Server**
   - Returns to the project root.
   - Starts the Wails development server.

### Notes

- The script uses sed commands which may need to be adjusted for different operating systems.
- It assumes that the specified package manager and required tools (like Wails CLI) are already installed.
- The script automatically starts the Wails development server after setup.

This script provides a quick and automated way to set up a project with Wails, Svelte, and shadcn-svelte, saving time on manual configuration and setup.
