<script lang="ts">
	import { invoke } from "@tauri-apps/api";

	let profiles: string[] = [];
	async function getProfiles(device: DeviceInfo) {
		profiles = JSON.parse(await invoke("get_profiles", { device: device.id }));
		value = profiles[0];
	}

	export let profile: Profile;
	async function setProfile(id: string) {
		if (!device || !id) return;
		await invoke("set_selected_profile", { device: device.id, profile: id });
		profile = JSON.parse(await invoke("get_selected_profile", { device: device.id }));
	}

	let value: string;
	$: setProfile(value);

	export let device: DeviceInfo;
	$: getProfiles(device);
</script>

<select bind:value class="mt-1 w-full">
	{#each profiles as profile}
		<option value={profile}> {profile} </option>
	{/each}
</select>
