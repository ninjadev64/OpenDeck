<script lang="ts">
	import type ActionList from "./ActionList.svelte";
	import type DeviceSelector from "./DeviceSelector.svelte";

	import ArrowClockwise from "phosphor-svelte/lib/ArrowClockwise";
	import CloudArrowDown from "phosphor-svelte/lib/CloudArrowDown";
	import FileArrowUp from "phosphor-svelte/lib/FileArrowUp";
	import Trash from "phosphor-svelte/lib/Trash";
	import ListedPlugin from "./ListedPlugin.svelte";
	import Popup from "./Popup.svelte";
	import Tooltip from "./Tooltip.svelte";

	import { localisations, settings } from "$lib/settings";

	import { invoke } from "@tauri-apps/api/core";
	import { onOpenUrl } from "@tauri-apps/plugin-deep-link";
	import { ask, message, open } from "@tauri-apps/plugin-dialog";

	export let actionList: () => ActionList;
	export let deviceSelector: () => DeviceSelector;

	let showPopup: boolean;

	async function installPlugin(id: string, name: string, url: string | null = null, file: string | null = null) {
		if (!file && !await ask(`Install "${name}"? It may take a while to download the plugin.`, { title: `Install "${name}"?` })) return;
		try {
			await invoke("install_plugin", { id, url, file });
			message(`Successfully installed "${name}".`, { title: `Installed "${name}"` });
			actionList().reload();
			installed = await invoke("list_plugins");
		} catch (error: any) {
			message(`Failed to install "${name}": ${error}`, { title: `Failed to install "${name}"` });
		}
	}

	let choices: any[] | undefined;
	let choice: number;
	let finishChoice = (_: unknown) => {};
	async function chooseAsset(assets: any[]): Promise<any> {
		choices = assets;
		await new Promise((resolve) => finishChoice = resolve);
		choices = undefined;
		return assets[choice];
	}

	type GitHubPlugin = {
		name: string;
		author: string;
		repository: string;
		download_url: string | undefined;
	};
	async function installPluginGitHub(id: string, plugin: GitHubPlugin) {
		if (plugin.download_url) {
			await installPlugin(id, plugin.name, plugin.download_url);
			return;
		}

		let endpoint = new URL(plugin.repository);
		endpoint.hostname = "api." + endpoint.hostname;
		endpoint.pathname = "/repos" + endpoint.pathname + "/releases";

		let res;
		try {
			res = await (await fetch(endpoint)).json();
		} catch (error: any) {
			message(`Failed to install "${plugin.name}": ${error}`, { title: `Failed to install "${plugin.name}"` });
			return;
		}

		let assets = [];
		for (const asset of res[0].assets) {
			if (asset.name.toLowerCase().endsWith(".streamdeckplugin") || asset.name.toLowerCase().endsWith(".zip")) {
				assets.push(asset);
			}
		}
		let selected;
		if (assets.length == 1) selected = assets[0];
		else selected = await chooseAsset(assets);

		await installPlugin(id, plugin.name, selected.browser_download_url);
	}

	async function installPluginElgato(plugin: any) {
		await installPlugin(plugin.id, plugin.name);
	}

	async function installPluginFile() {
		const path = await open({ multiple: false, directory: false });
		const id = path ? prompt("Plugin ID:") : null;
		if (!id || id.split(".").length < 3) {
			message("Failed to install plugin from file: invalid plugin ID", { title: "Failed to install" });
			return;
		}
		installPlugin(id, id, null, path);
	}

	async function removePlugin(plugin: any) {
		if (!await ask(`Are you sure you want to remove "${plugin.name}"?`, { title: `Remove "${plugin.name}"?` })) return;
		try {
			await invoke("remove_plugin", { id: plugin.id });
			message(`Successfully removed "${plugin.name}".`, { title: `Removed "${plugin.name}"` });
			actionList().reload();
			deviceSelector().reloadProfiles();
			installed = await invoke("list_plugins");
		} catch (error: any) {
			message(`Failed to remove "${plugin.name}": ${error}`, { title: `Failed to remove "${plugin.name}"` });
		}
	}

	let installed: any[] = [];
	(async () => installed = await invoke("list_plugins"))();

	let plugins: { [id: string]: GitHubPlugin };
	(async () => plugins = await (await fetch("https://ninjadev64.github.io/openaction-plugins/catalogue.json")).json())();

	let search: string = "";

	onOpenUrl((urls: string[]) => {
		if (!urls[0].includes("installPlugin/")) return;
		let id = urls[0].split("installPlugin/")[1];
		if (!plugins[id]) return;
		installPluginGitHub(id, plugins[id]);
	});
</script>

<button
	class="mt-2 p-1 w-1/2 text-sm text-neutral-700 dark:text-neutral-300 bg-neutral-100 dark:bg-neutral-700 border dark:border-neutral-600 rounded-lg outline-none"
	on:click={() => showPopup = true}
>
	Plugins
</button>

<svelte:window
	on:keydown={(event) => {
		if (event.key == "Escape") showPopup = false;
	}}
/>

<Popup show={showPopup}>
	<button class="mr-2 my-1 float-right text-xl dark:text-neutral-300" on:click={() => showPopup = false}>✕</button>
	<h2 class="m-2 font-semibold text-xl dark:text-neutral-300">Manage plugins</h2>

	<h2 class="mx-2 mt-6 mb-2 text-lg dark:text-neutral-400">Installed plugins</h2>
	<div class="grid grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
		{#each installed as plugin}
			<ListedPlugin
				icon="http://localhost:57118/{plugin.icon}"
				name={($localisations && $localisations[plugin.id] && $localisations[plugin.id].Name) ? $localisations[plugin.id].Name : plugin.name}
				subtitle={plugin.version}
				action={() => {
					if ($settings?.developer) invoke("reload_plugin", { id: plugin.id });
					else removePlugin(plugin);
				}}
			>
				{#if $settings?.developer}
					<ArrowClockwise
						size="24"
						color={document.documentElement.classList.contains("dark") ? "#C0BFBC" : "#77767B"}
					/>
				{:else}
					<Trash
						size="24"
						color={document.documentElement.classList.contains("dark") ? "#C0BFBC" : "#77767B"}
					/>
				{/if}
			</ListedPlugin>
		{/each}
	</div>

	<div class="flex flex-row justify-between items-center mx-2 mt-6 mb-2">
		<h2 class="text-lg dark:text-neutral-400">Plugin store</h2>
		<button
			class="flex flex-row items-center mt-2 px-1 py-0.5 text-sm text-neutral-700 dark:text-neutral-300 bg-neutral-100 dark:bg-neutral-700 border dark:border-neutral-600 rounded-lg outline-none"
			on:click={installPluginFile}
		>
			<FileArrowUp />
			<span class="ml-1">Install from file</span>
		</button>
	</div>
	<div class="flex flex-row m-2">
		<input
			bind:value={search}
			class="grow p-2 dark:text-neutral-300 dark:bg-neutral-700 rounded-md outline-none"
			placeholder="Search plugins"
			type="search"
			spellcheck="false"
		/>
	</div>

	{#if !plugins}
		<h2 class="mx-2 mt-6 mb-2 text-md dark:text-neutral-400">Loading open-source plugin list...</h2>
	{:else}
		<div class="flex flex-row items-center mt-6 mb-2">
			<h2 class="mx-2 font-semibold text-md dark:text-neutral-400">Open-source plugins</h2>
			<Tooltip> Open-source plugins downloaded from the author's releases. </Tooltip>
		</div>
		<div class="grid grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
			{#each Object.entries(plugins) as [id, plugin]}
				<ListedPlugin
					icon="https://ninjadev64.github.io/openaction-plugins/icons/{id}.png"
					name={plugin.name}
					subtitle={plugin.author}
					hidden={!plugin.name.toUpperCase().includes(search.toUpperCase())}
					action={() => installPluginGitHub(id, plugin)}
				>
					<CloudArrowDown
						size="24"
						color={document.documentElement.classList.contains("dark") ? "#C0BFBC" : "#77767B"}
					/>
				</ListedPlugin>
			{/each}
		</div>
	{/if}

	{#if choices}
		<div
			class="fixed left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 mt-2 p-2 w-96 text-xs dark:text-neutral-300 bg-neutral-100 dark:bg-neutral-700 border-2 dark:border-neutral-600 rounded-lg z-40"
		>
			<h3 class="mb-2 font-semibold text-lg text-center">Choose a release asset</h3>
			<div class="select-wrapper">
				<select class="w-full" bind:value={choice}>
					{#each choices as choice, i}
						<option value={i}>{choice.name}</option>
					{/each}
				</select>
			</div>
			<button
				class="mt-2 p-1 w-full text-sm text-neutral-700 dark:text-neutral-300 bg-neutral-200 dark:bg-neutral-800 border dark:border-neutral-600 rounded-lg"
				on:click={finishChoice}
			>
				Install
			</button>
		</div>
	{/if}

	{#await fetch("https://plugins.amankhanna.me/catalogue.json")}
		<h2 class="mx-2 mt-6 mb-2 text-md dark:text-neutral-400">Loading plugin list...</h2>
	{:then res}
		{#await res.json() then entries}
			<div class="flex flex-row items-center mt-6 mb-2">
				<h2 class="mx-2 font-semibold text-md dark:text-neutral-400">Elgato App Store archive</h2>
				<Tooltip> Plugins archived from the now deprecated Elgato App Store. </Tooltip>
			</div>
			<div class="grid grid-cols-2 lg:grid-cols-3 xl:grid-cols-4">
				{#each entries as plugin}
					<ListedPlugin
						icon="https://plugins.amankhanna.me/icons/{plugin.id}.png"
						name={plugin.name}
						subtitle={plugin.author}
						hidden={!plugin.name.toUpperCase().includes(search.toUpperCase())}
						action={() => installPluginElgato(plugin)}
					>
						<CloudArrowDown
							size="24"
							color={document.documentElement.classList.contains("dark") ? "#C0BFBC" : "#77767B"}
						/>
					</ListedPlugin>
				{/each}
			</div>
		{/await}
	{/await}
</Popup>
