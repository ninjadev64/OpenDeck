<script lang="ts">
	import { createEventDispatcher } from "svelte";
	const dispatch = createEventDispatcher();

	import { invoke } from "@tauri-apps/api";

	let devices: DeviceInfo[] = [];
	
	async function refreshDevices() {
		devices = await invoke("get_devices");
	}
	
	let value: string;
	function change() {
		if (value == "refresh_devices") {
			refreshDevices();
			value = "placeholder";
		} else {
			dispatch("select", devices[parseInt(value)]);
		}
	}
</script>

<select bind:value on:change={change} class="w-full">
	<option value="placeholder" disabled selected> Choose a device... </option>

	{#each devices as device, index}
		<option value={index}> {device.name} </option>
	{/each}
	
	<option value="refresh_devices"> Refresh devices </option>
</select>
