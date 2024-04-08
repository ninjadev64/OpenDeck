<script lang="ts">
	import type { DeviceInfo } from "$lib/DeviceInfo";
	import type { Profile } from "$lib/Profile";

	import Popup from "./Popup.svelte";

	import { invoke } from "@tauri-apps/api";

	let profiles: string[] = [];
	async function getProfiles(device: DeviceInfo) {
		profiles = await invoke("get_profiles", { device: device.id });
		profile = await invoke("get_selected_profile", { device: device.id });
		if (value == profile.id) return;
		value = profile.id;
		oldValue = value;
	}

	export let profile: Profile;
	async function setProfile(id: string) {
		if (!device || !id) return;
		await invoke("set_selected_profile", { device: device.id, id });
		profile = await invoke("get_selected_profile", { device: device.id });
		if (!profiles.includes(id)) profiles = [ ...profiles, id ];
	}

	async function deleteProfile(id: string) {
		await invoke("delete_profile", { device: device.id, profile: id });
		profiles.splice(profiles.indexOf(id), 1);
		profiles = profiles;
	}

	let oldValue: string;
	let value: string;
	$: {
		if (value == "opendeck_edit_profiles") {
			if (oldValue) showPopup = true;
			value = oldValue;
		} else if (value && (!profile || profile.id != value)) {
			setProfile(value);
			oldValue = value;
		}
	}

	export let device: DeviceInfo;
	$: getProfiles(device);

	let showPopup = false;
	let nameInput: string;
</script>

<select bind:value class="mt-1 w-full">
	{#each profiles as profile}
		<option value={profile}> {profile} </option>
	{/each}
	<option value="opendeck_edit_profiles"> Edit... </option>
</select>

<Popup show={showPopup}>
	<button class="mr-1 float-right text-xl" on:click={() => showPopup = false}> ✕ </button>
	<h2 class="text-xl font-semibold"> {device.name} </h2>

	<div class="flex flex-row mt-2 mb-1">
		<input class="grow p-2 rounded-l-md outline-none" placeholder="Profile name" bind:value={nameInput} />
		<button class="px-4 bg-gray-200 rounded-r-md" on:click={async () => {
			await setProfile(nameInput);
			value = nameInput;
			nameInput = "";
			showPopup = false;
		}}> Create </button>
	</div>

	<div class="space-y-2 divide-y">
		{#each profiles as profile, i}
			<div class="pt-2">
				<input type="radio" bind:group={value} value={profile} />
				{profile}
				{#if profile != value}
					<!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
					<img
						src="/rubbish.png"
						class="float-right w-6 cursor-pointer"
						alt="Remove profile"
						on:click={() => deleteProfile(profile)}
						on:keyup={() => deleteProfile(profile)}
					/>
				{/if}
			</div>
		{/each}
	</div>
</Popup>
