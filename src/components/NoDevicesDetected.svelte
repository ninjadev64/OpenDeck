<script lang="ts">
	import { invoke } from "@tauri-apps/api/core";
</script>

<div class="flex flex-col justify-center items-center w-full h-full text-center dark:text-neutral-300">
	<div class="w-80 text-sm">
		<h2 class="text-lg font-bold mb-2"> No devices detected </h2>
		<p class="mb-2"> Make sure your devices are connected properly and you have permission to access them. </p>
		<p class="mb-2"> On GNU+Linux, ensure you have the correct udev subsystem rules installed. </p>
		<p class="mb-3"> On Windows, ensure you have the correct drivers for your device installed. </p>
		<button
			class="px-2 py-1 text-sm text-neutral-700 dark:text-neutral-300 bg-neutral-100 dark:bg-neutral-700 border dark:border-neutral-600 rounded-lg"
			on:click={() => invoke("rescan_devices")}
		>
			Rescan
		</button>
	</div>
</div>
