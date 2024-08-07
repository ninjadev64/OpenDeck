<script lang="ts">
	import type { DeviceInfo } from "$lib/DeviceInfo";
	import type { Profile } from "$lib/Profile";

	import Download from "phosphor-svelte/lib/Download";
	import Trash from "phosphor-svelte/lib/Trash";
	import Upload from "phosphor-svelte/lib/Upload";
	import Popup from "./Popup.svelte";

	import { invoke } from "@tauri-apps/api";

	let folders: { [name: string]: string[] } = {};
	let value: string;
	async function getProfiles(device: DeviceInfo) {
		let profiles: string[] = await invoke("get_profiles", { device: device.id });
		folders = {};
		for (const id of profiles) {
			let folder = id.includes("/") ? id.split("/")[0] : "";
			if (folders[folder]) folders[folder].push(id);
			else folders[folder] = [ id ];
		}
		profile = await invoke("get_selected_profile", { device: device.id });
		value = profile.id;
		oldValue = value;
	}

	export let device: DeviceInfo;
	getProfiles(device);

	export let profile: Profile;
	export async function setProfile(id: string, toSet: Profile | undefined = undefined) {
		if (!device || !id) return;
		if (value != id) {
			value = id;
			return;
		}
		await invoke("set_selected_profile", { device: device.id, id, profile: toSet });
		profile = await invoke("get_selected_profile", { device: device.id });

		let folder = id.includes("/") ? id.split("/")[0] : "";
		if (folders[folder]) {
			if (!folders[folder].includes(id)) folders[folder].push(id);
		}
		else folders[folder] = [ id ];
		folders = folders;
	}

	async function deleteProfile(id: string) {
		await invoke("delete_profile", { device: device.id, profile: id });
		let folder = id.includes("/") ? id.split("/")[0] : "";
		folders[folder].splice(folders[folder].indexOf(id), 1);
		folders = folders;
	}

	function exportProfile() {
		navigator.clipboard.writeText(JSON.stringify(
			profile,
			(k: string, v: string) => {
				if (k == "context") return v.replace(device.id, "no-device");
				return v;
			},
			2
		));
		alert(`Copied profile ${value} to clipboard.`);
	}

	let fileInput: HTMLInputElement;
	function importProfile() {
		if (!fileInput.files || fileInput.files.length == 0) return;
		const reader = new FileReader();
		reader.onload = async () => {
			let p: Profile = JSON.parse(
				reader.result! as string,
				(k: string, v: string) => {
					if (k == "context") return v.replace("no-device", device.id);
					return v;
				}
			);
			if (p.keys.length != device.rows * device.columns || p.sliders.length != device.sliders) {
				alert("The imported profile is incompatible with the selected device.");
				return;
			}
			if ((p.id.includes("/") ? folders[p.id.split("/")[0]] : folders[""]).includes(p.id)) {
				if (!await confirm(`Are you sure you want to overwrite profile ${p.id}?`)) return;
			}
			setProfile(p.id, p);
			value = p.id;
		};
		reader.readAsText(fileInput.files[0]);
	}

	let oldValue: string;
	$: {
		if (value == "opendeck_edit_profiles") {
			if (oldValue) showPopup = true;
			value = oldValue;
		} else if (value && (!profile || profile.id != value)) {
			setProfile(value);
			oldValue = value;
		}
	}

	let showPopup: boolean;
	let nameInput: HTMLInputElement;
</script>

<div class="select-wrapper">
	<select bind:value class="mt-1 w-full">
		{#each Object.entries(folders) as [ id, profiles ]}
			{#if id && profiles.length}
				<optgroup label={id}>
					{#each profiles as profile}
						<option value={profile}> {profile.split("/")[1]} </option>
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

		<input
			type="file"
			bind:this={fileInput}
			on:change={importProfile}
			class="hidden"
		/>
		<button
			on:click={() => fileInput.click()}
			class="px-3 dark:text-neutral-300 bg-neutral-200 dark:bg-neutral-900 border-r border-neutral-300 dark:border-neutral-700"
		>
			<Upload
				size="20"
				color={document.documentElement.classList.contains("dark") ? "#C0BFBC" : "#77767B"}
			/>
		</button>

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
					<input type="radio" bind:group={value} value="{profile}" />
					<span class="dark:text-neutral-400"> {id ? profile.split("/")[1] : profile} </span>
					{#if profile != value}
						<button
							on:click={() => deleteProfile(profile)}
							class="float-right"
						>
							<Trash
								size="20"
								color={document.documentElement.classList.contains("dark") ? "#C0BFBC" : "#77767B"}
							/>
						</button>
					{:else}
						<button
							on:click={exportProfile}
							class="float-right"
						>
							<Download
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
