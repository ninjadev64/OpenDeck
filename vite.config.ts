import { sveltekit } from "@sveltejs/kit/vite";
import { defineConfig } from "vite";

export default defineConfig({
	plugins: [sveltekit()],
	clearScreen: false,
	server: {
		watch: {
			ignored: ["**/src-tauri/**"],
		},
	},
});
