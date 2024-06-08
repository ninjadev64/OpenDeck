<script lang="ts">
	import type { DeviceInfo } from "$lib/DeviceInfo";
	import type { Profile } from "$lib/Profile";

	import { inspectedMultiAction } from "$lib/propertyInspector";

	import ActionList from "../components/ActionList.svelte";
	import DeviceSelector from "../components/DeviceSelector.svelte";
	import DeviceView from "../components/DeviceView.svelte";
	import MultiActionView from "../components/MultiActionView.svelte";
	import NoDevicesDetected from "../components/NoDevicesDetected.svelte";
	import PluginManager from "../components/PluginManager.svelte";
	import ProfileSelector from "../components/ProfileSelector.svelte";
	import PropertyInspectorView from "../components/PropertyInspectorView.svelte";
	import SettingsView from "../components/SettingsView.svelte";
	import {invoke} from "@tauri-apps/api";
	import {listen} from "@tauri-apps/api/event";
	import {dev} from "$app/environment";

	let selectedDevice: string;
	let devices: { [id: string]: DeviceInfo } = {};
	let selectedProfile: { [id: string]: Profile } = {};
	let actionList: ActionList;
	let profileSelector: ProfileSelector;

</script>

<div class="flex flex-col grow">
	{#if $inspectedMultiAction}
		<MultiActionView bind:profile={selectedProfile[selectedDevice]} />
	{:else if Object.keys(devices).length > 0 && selectedProfile}
		{#each Object.entries(devices).sort() as [ id, device ]}
			{#if device && selectedProfile[id]}
				<div class:hidden={id !== selectedDevice}>
					<DeviceView bind:device={device} bind:profile={selectedProfile[id]}  />
				</div>

			{/if}
		{/each}
	{:else}
		<NoDevicesDetected />
	{/if}

	{#if selectedProfile[selectedDevice]}
		<PropertyInspectorView bind:device={devices[selectedDevice]} bind:profile={selectedProfile[selectedDevice]} />
	{/if}
</div>

<div class="flex flex-col p-2 grow max-w-[18rem] h-full border-l dark:border-neutral-700">
	{#if !$inspectedMultiAction}
		<DeviceSelector bind:value={selectedDevice} bind:devices={devices} bind:selectedProfile={selectedProfile}/>
		{#if selectedDevice && devices[selectedDevice]}
			<ProfileSelector
				bind:device={devices[selectedDevice]}
				bind:profile={selectedProfile[selectedDevice]}
				bind:this={profileSelector}
			/>
		{/if}
	{/if}
	<ActionList bind:this={actionList} />
	<hr class="mt-2 border dark:border-neutral-700" />
	<div class="flex flex-row">
		<PluginManager bind:actionList bind:profileSelector />
		<SettingsView />
	</div>
</div>
