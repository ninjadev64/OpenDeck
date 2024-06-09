<script lang="ts">
	import type { DeviceInfo } from "$lib/DeviceInfo";
	import type { Profile } from "$lib/Profile";
	import type ProfileSelector from "./ProfileSelector.svelte";

	import { invoke } from "@tauri-apps/api";
	import { listen } from "@tauri-apps/api/event";

	export let devices: { [id: string]: DeviceInfo } = {};
	export let value: string;
	export let selectedProfiles: { [id: string]: Profile } = {};

	let registered: string[] = [];
	$: {
		if (!value || !devices[value]) value = Object.keys(devices).sort()[0];
		for (const [ id, device ] of Object.entries(devices)) {
			if (!registered.includes(id)) {
				(async () => {
					let profile: Profile = await invoke("get_selected_profile", { device: device.id });
					selectedProfiles[id] = profile;
					await invoke("set_selected_profile", { device: id, id: profile.id });
				})();
				registered.push(id);
			}
		}
	}

	export function reloadProfiles() {
		registered = [];
	}

	export let profileSelector: () => ProfileSelector;
	listen("switch_profile", async ({ payload }: { payload: { device: string, profile: string }}) => {
		if (payload.device == value) {
			profileSelector().setProfile(payload.profile);
		} else {
			await invoke("set_selected_profile", { device: payload.device, id: payload.profile });
			selectedProfiles[payload.device] = await invoke("get_selected_profile", { device: payload.device });
		}
	});

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
