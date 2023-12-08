<script lang="ts">
	import { invoke } from "@tauri-apps/api";

	let devices: { [id: string]: DeviceInfo } = {};

	async function refreshDevices() {
		devices = JSON.parse(await invoke("get_devices"));
	}
	setTimeout(refreshDevices, 5e2);

	let value: string;
	export let device: DeviceInfo | null;
	function change() {
		if (value == "refresh_devices") {
			refreshDevices();
			value = "placeholder";
		} else {
			device = devices[value];
		}
	}
</script>

<select bind:value on:change={change} class="w-full">
	<option value="placeholder" disabled selected> Choose a device... </option>

	{#each Object.entries(devices) as [ id, device ]}
		<option value={id}> {device.name} </option>
	{/each}

	<option value="refresh_devices"> Refresh devices </option>
</select>
