<script lang="ts">
    import { settings } from "$lib/settings";
    import { invoke } from "@tauri-apps/api";

	import Popup from "./Popup.svelte";
	import Tooltip from "./Tooltip.svelte";

	let showPopup: boolean;
	let buildInfo: string;
	(async () => buildInfo = await invoke("get_build_info"))();

	settings.subscribe((settings) => {
		if (settings) updateTheme(settings.darktheme);
	});

	function updateTheme(darktheme: boolean) {
		if (darktheme) {
			document.documentElement.classList.add("dark");
		} else {
			document.documentElement.classList.remove("dark");
		}
	}
</script>

<button
	class="ml-2 mt-2 p-1 w-1/2 text-sm text-neutral-700 dark:text-neutral-300 bg-neutral-100 dark:bg-neutral-700 border dark:border-neutral-600 rounded-lg"
	on:click={() => showPopup = true}
>
	Settings
</button>

<Popup show={showPopup}>
	<button class="mr-2 my-1 float-right text-xl dark:text-neutral-300" on:click={() => showPopup = false}> ✕ </button>
	<h2 class="m-2 font-semibold text-xl dark:text-neutral-300"> Settings </h2>
	{#if $settings}
		<div class="flex flex-row items-center m-2 space-x-2">
			<span class="dark:text-neutral-400"> Language: </span>
			<div class="select-wrapper">
				<select bind:value={$settings.language} class="w-32">
					<option value="en"> English </option>
					<option value="es"> Español </option>
					<option value="zh_CN"> 中文 </option>
					<option value="fr"> Français </option>
					<option value="de"> Deutsch </option>
					<option value="ja"> 日本語 </option>
					<option value="ko"> 韓国語 </option>
				</select>
			</div>
			<Tooltip>
				OpenDeck itself is not yet translated.
				Changing this setting will translate the text from installed plugins into your language for those that support it.
			</Tooltip>
		</div>

		<div class="flex flex-row items-center m-2 space-x-2">
			<span class="dark:text-neutral-400"> Autolaunch: </span>
			<input type="checkbox" bind:checked={$settings.autolaunch} />
			<Tooltip> If this option is enabled, OpenDeck will automatically start at login. </Tooltip>
		</div>

		<div class="flex flex-row items-center m-2 space-x-2">
			<span class="dark:text-neutral-400"> Dark theme: </span>
			<input type="checkbox" bind:checked={$settings.darktheme} />
		</div>

		<div class="flex flex-row items-center m-2 space-x-2">
			<span class="dark:text-neutral-400"> Default Device brightness: </span>
			<input type="range" id="brightness" name="points" min="0" max="100" bind:value={$settings.brightness} />
		</div>
	{/if}

	<div class="ml-2">
		<button
			class="mt-2 mb-4 px-2 py-1 text-sm text-neutral-700 dark:text-neutral-300 bg-neutral-100 dark:bg-neutral-700 border dark:border-neutral-600 rounded-lg"
			on:click={() => invoke("open_config_directory")}
		>
			Open config directory
		</button>
		<span class="text-xs dark:text-neutral-400">
			{@html buildInfo}
		</span>
	</div>
</Popup>
