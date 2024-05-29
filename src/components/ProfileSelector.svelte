<script lang="ts">
	import type { DeviceInfo } from "$lib/DeviceInfo";
	import type { Profile } from "$lib/Profile";

	import Trash from "phosphor-svelte/lib/Trash";
	import Popup from "./Popup.svelte";

	import { invoke } from "@tauri-apps/api";
	import { listen } from "@tauri-apps/api/event";

	let folders: { [name: string]: string[] } = { "": [] };
	async function getProfiles(device: DeviceInfo) {
		let profiles: string[] = await invoke("get_profiles", { device: device.id });
		folders = { "": [] };
		for (const id of profiles) {
			if (id.includes("/")) {
				let folder = id.split("/")[0];
				let name = id.split("/")[1];
				if (folders[folder]) folders[folder].push(name);
				else folders[folder] = [ name ];
			} else {
				folders[""].push(id);
			}
		}
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
		if (id.includes("/")) {
			let folder = id.split("/")[0];
			let name = id.split("/")[1];
			if (folders[folder] && !folders[folder].includes(name)) folders[folder].push(name);
			else folders[folder] = [ name ];
		} else {
			if (!folders[""].includes(id)) folders[""].push(id);
		}
	}

	async function deleteProfile(id: string) {
		await invoke("delete_profile", { device: device.id, profile: id });
		if (id.includes("/")) {
			let folder = id.split("/")[0];
			folders[folder].splice(folders[folder].indexOf(id.split("/")[1]), 1);
		} else {
			folders[""].splice(folders[""].indexOf(id), 1);
		}
		folders = folders;
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

	let showPopup: boolean;
	let nameInput: HTMLInputElement;

	listen("switch_profile", async ({ payload }: { payload: { device: string, profile: string }}) => {
		if (payload.device == device.id) {
			value = payload.profile;
		}
	});
</script>

<div class="select-wrapper">
	<select bind:value class="mt-1 w-full">
		{#each Object.entries(folders) as [ id, profiles ]}
			{#if id && profiles.length}
				<optgroup label={id}>
					{#each profiles as profile}
						<option value={`${id}/${profile}`}> {profile} </option>
					{/each}
				</optgroup>
			{:else}
				{#each profiles as profile}
					<option value={profile}> {profile} </option>
				{/each}
			{/if}
		{/each}
		<option value="opendeck_edit_profiles"> Edit... </option>
	</select>
</div>

<Popup show={showPopup}>
	<button class="mr-1 float-right text-xl dark:text-neutral-300" on:click={() => showPopup = false}> ✕ </button>
	<h2 class="text-xl font-semibold dark:text-neutral-300"> {device.name} </h2>

	<div class="flex flex-row mt-2 mb-1">
		<input
			bind:this={nameInput}
			pattern="[a-zA-Z0-9_ ]+(\/[a-zA-Z0-9_ ]+)?"
			class="grow p-2 dark:text-neutral-300 invalid:text-red-400 dark:bg-neutral-700 rounded-l-md outline-none"
			placeholder='Profile ID (e.g. "folder/profile")'
		/>
		<button
			on:click={async () => {
				if (!nameInput.checkValidity()) return;
				await setProfile(nameInput.value);
				value = nameInput.value;
				nameInput.value = "";
				showPopup = false;
			}}
			class="px-4 dark:text-neutral-300 bg-neutral-200 dark:bg-neutral-900 rounded-r-md"
		>
			Create
		</button>
	</div>

	<div class="divide-y">
		{#each Object.entries(folders) as [ id, profiles ]}
			{#if id && profiles.length}
				<h4 class="py-2 font-bold text-lg dark:text-neutral-300"> {id} </h4>
			{/if}
			{#each profiles as profile}
				<div class="py-2" class:ml-6={id} class:pl-2={id}>
					<input type="radio" bind:group={value} value="{(id ? id + "/" : "") + profile}" />
					<span class="dark:text-neutral-400"> {profile} </span>
					{#if (id ? id + "/" : "") + profile != value}
						<!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
						<button
							class="float-right"
							on:click={() => deleteProfile((id ? id + "/" : "") + profile)}
							on:keyup={() => deleteProfile((id ? id + "/" : "") + profile)}
						>
							<Trash
								size="20"
								color={document.documentElement.classList.contains("dark") ? "#C0BFBC" : "#77767B"}
							/>
						</button>
					{/if}
				</div>
			{/each}
		{/each}
	</div>
</Popup>
