<script lang="ts">
    import { settings } from "$lib/settings";

	import Popup from "./Popup.svelte";
	import Tooltip from "./Tooltip.svelte";

	let showPopup = false;
</script>

<button
	class="p-1 ml-2 mt-2 w-1/2 text-sm text-gray-700 bg-gray-100 border rounded-lg"
	on:click={() => showPopup = true}
>
	Settings
</button>

<Popup show={showPopup}>
	<button class="mr-2 my-1 float-right text-xl" on:click={() => showPopup = false}> ✕ </button>
	<h2 class="m-2 font-semibold text-xl"> Settings </h2>
	{#if $settings}
		<div class="flex flex-row items-center m-2 space-x-2">
			<span> Language: </span>
			<select bind:value={$settings.language}>
				<option value="en"> English </option>
				<option value="es"> Español </option>
				<option value="zh_CN"> 中文 </option>
				<option value="fr"> Français </option>
				<option value="de"> Deutsch </option>
				<option value="ja"> 日本語 </option>
				<option value="ko"> 韓国語 </option>
			</select>
			<Tooltip>
				OpenDeck itself is not yet translated.
				Changing this setting will translate the text from installed plugins into your language for those that support it.
			</Tooltip>
		</div>
	{/if}
</Popup>
