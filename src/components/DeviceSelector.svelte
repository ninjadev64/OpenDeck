<script lang="ts">
	import type { DeviceInfo } from "$lib/DeviceInfo";
	import type { Profile } from "$lib/Profile";

	import { invoke } from "@tauri-apps/api";
	import { listen } from "@tauri-apps/api/event";

	export let devices: { [id: string]: DeviceInfo } = {};
	let registered: string[] = [];
	export let value: string;
	let device: DeviceInfo | null = null;

	export let selectedProfile: { [id: string]: Profile } = {};

	$: {
		if (!value || !devices[value]) value = Object.keys(devices).sort()[0];
		device = devices[value];
		for (const [ id, device ] of Object.entries(devices)) {
			if (!registered.includes(id)) {
				(async () => {
					let profile: Profile = await invoke("get_selected_profile", { device: device.id });
					selectedProfile[id] = profile;
					await invoke("set_selected_profile", { device: id, id: profile.id });
				})();
				registered.push(id);
			}
		}
	}

	(async () => devices = await invoke("get_devices"))();
	listen("devices", ({ payload }: { payload: { [id: string]: DeviceInfo }}) => devices = payload);
</script>

<div class="select-wrapper">
	<select bind:value class="w-full">
		<option value="" disabled selected> Choose a device... </option>

		{#each Object.entries(devices).sort() as [ id, device ]}
			<option value={id}> {device.name} </option>
		{/each}
	</select>
</div>
