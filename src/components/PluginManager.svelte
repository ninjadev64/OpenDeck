<script lang="ts">
	import Popup from "./Popup.svelte";

	import { localisations } from "$lib/settings";
	import { invoke } from "@tauri-apps/api";

	import type ActionList from "./ActionList.svelte";
	export let actionList: ActionList;

	let showPopup = false;

	async function installPlugin(plugin: any) {
		if (!await confirm(`Install ${plugin.name}? It may take a while to download the plugin.`)) {
			return;
		}
		try {
			await invoke("install_plugin", { id: plugin.id });
			alert(`Successfully installed ${plugin.name}`);
			actionList.reload();
			plugins = await invoke("list_plugins");
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

	let search: string = "";
</script>

<button
	class="mt-2 p-1 w-1/2 text-sm text-gray-700 bg-gray-100 border rounded-lg"
	on:click={() => showPopup = true}
>
	Plugins
</button>

<Popup show={showPopup}>
	<button class="mr-2 my-1 float-right text-xl" on:click={() => showPopup = false}> ✕ </button>
	<h2 class="m-2 font-semibold text-xl"> Manage plugins </h2>

	<h2 class="mx-2 mt-6 mb-2 text-lg"> Installed plugins </h2>
	<div class="grid grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
		{#each plugins as plugin}
			<div class="flex flex-row items-center m-2 p-2 bg-gray-200 rounded-md">
				<img src={"http://localhost:57118/" + plugin.icon} class="w-24 rounded-md" alt={plugin.name} />
				<div class="ml-4 mr-2">
					<p>
						<span class="font-semibold">
							{($localisations && $localisations[plugin.id] && $localisations[plugin.id].Name) ? $localisations[plugin.id].Name : plugin.name}
						</span>
						{plugin.version}
					</p>
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
		<h2 class="mx-2 mt-6 mb-2 text-md"> Loading plugin list... </h2>
	{:then res}
		{#await res.json() then entries}
			<h2 class="mx-2 mt-6 mb-2 text-lg"> Plugin store </h2>
			<div class="flex flex-row m-2">
				<input
					bind:value={search}
					class="grow p-2 rounded-md outline-none"
					placeholder="Search plugins"
					type="search"
					spellcheck="false"
				/>
			</div>
			<div class="grid grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
				{#each entries as plugin}
					<div
						class="flex flex-row items-center m-2 p-2 bg-gray-200 rounded-md"
						class:hidden={!plugin.name.toUpperCase().includes(search.toUpperCase())}
					>
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
