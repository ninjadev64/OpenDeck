<script lang="ts">
	import Popup from "./Popup.svelte";

    import { invoke } from "@tauri-apps/api";

	let showPopup = false;

	async function installPlugin(plugin: any) {
		if (!await confirm(
			`Install ${plugin.name}? It may take a while to download the plugin.\n`+
			"You will need to relaunch OpenDeck for the plugin to be available."
		)) {
			return;
		}
		try {
			await invoke("install_plugin", { id: plugin.id });
			alert(`Successfully installed ${plugin.name}`);
		} catch (error: any) {
			alert(`Failed to install ${plugin.name}: ${error.description}`);
		}
	}

	async function removePlugin(plugin: any) {
		if (!await confirm(`Are you sure you want to remove ${plugin.name}? This action will relaunch OpenDeck.`)) return;
		try {
			await invoke("remove_plugin", { id: plugin.id });
			alert(`Successfully removed ${plugin.name}`);
		} catch (error: any) {
			alert(`Failed to remove ${plugin.name}: ${error.description}`);
		}
	}

	let plugins: any[] = [];
	(async () => plugins = await invoke("list_plugins"))();
</script>

<hr class="mt-2 border" />
<button
	class="p-1 mt-2 w-full text-sm text-gray-700 bg-gray-100 border rounded-lg"
	on:click={() => showPopup = true}
>
	Get plugins
</button>

<Popup show={showPopup}>
	<button class="mr-2 my-1 float-right text-xl" on:click={() => showPopup = false}> ✕ </button>
	<h2 class="m-2 font-semibold text-xl"> Manage plugins </h2>

	<h2 class="mx-2 mt-6 mb-2 text-lg"> Installed plugins </h2>
	<div class="grid grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
		{#each plugins as plugin}
			<div class="flex flex-row inline items-center m-2 p-2 bg-gray-200 rounded-md">
				<img src={"http://localhost:57118" + plugin.icon} class="w-24 rounded-md" alt={plugin.name} />
				<div class="ml-4 mr-2">
					<p><span class="font-semibold"> {plugin.name} </span>{plugin.version}</p>
					{plugin.author}
				</div>

				<!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
				<img
					src="/rubbish.png"
					class="ml-auto mr-4 w-6 cursor-pointer"
					alt="Remove plugin"
					on:click={() => removePlugin(plugin)} on:keyup={() => removePlugin(plugin)}
				/>
			</div>
		{/each}
	</div>

	{#await fetch("https://plugins.amansprojects.com/catalogue.json")}
		Loading plugin list...
	{:then res}
		{#await res.json() then entries}
			<h2 class="mx-2 mt-6 mb-2 text-lg"> Plugin store </h2>
			<div class="grid grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
				{#each entries as plugin}
					<div class="flex flex-row inline items-center m-2 p-2 bg-gray-200 rounded-md">
						<img src="https://plugins.amansprojects.com/icons/{plugin.id}.png" class="w-24 rounded-md" alt={plugin.name} />
						<div class="ml-4 mr-2">
							<p class="font-semibold"> {plugin.name} </p>
							{plugin.author}
						</div>

						<!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
						<img
							src="/download.png"
							class="ml-auto mr-4 w-6 cursor-pointer"
							alt="Install plugin"
							on:click={() => installPlugin(plugin)} on:keyup={() => installPlugin(plugin)}
						/>
					</div>
				{/each}
			</div>
		{/await}
	{/await}
</Popup>