import { sveltekit } from "@sveltejs/kit/vite";
import { defineConfig } from "vite";

export default defineConfig({
	plugins: [ sveltekit() ],
	server: {
		watch: {
			usePolling: true,
			ignored: [ "src-tauri/target/**" ]
		}
	}
});
