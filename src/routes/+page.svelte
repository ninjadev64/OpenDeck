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

	let devices: { [id: string]: DeviceInfo } = {};
	let selectedDevice: string;
	let selectedProfiles: { [id: string]: Profile } = {};

	let actionList: ActionList;
	let profileSelector: ProfileSelector;
</script>

<div class="flex flex-col grow">
	{#if $inspectedMultiAction}
		<MultiActionView bind:profile={selectedProfiles[selectedDevice]} />
	{:else if Object.keys(devices).length > 0 && selectedProfiles}
		{#each Object.entries(devices) as [ id, device ]}
			{#if device && selectedProfiles[id]}
				<DeviceView bind:device={device} bind:profile={selectedProfiles[id]} bind:selectedDevice={selectedDevice} />
			{/if}
		{/each}
	{:else}
		<NoDevicesDetected />
	{/if}

	{#if selectedProfiles[selectedDevice]}
		<PropertyInspectorView bind:device={devices[selectedDevice]} bind:profile={selectedProfiles[selectedDevice]} />
	{/if}
</div>

<div class="flex flex-col p-2 grow max-w-[18rem] h-full border-l dark:border-neutral-700">
	{#if !$inspectedMultiAction}
		<DeviceSelector bind:devices={devices} bind:value={selectedDevice} bind:selectedProfiles={selectedProfiles} />
		{#if selectedDevice && devices[selectedDevice]}
			<ProfileSelector
				bind:device={devices[selectedDevice]}
				bind:profile={selectedProfiles[selectedDevice]}
				bind:this={profileSelector}
			/>
		{/if}
	{/if}
	<ActionList bind:this={actionList} />
	<hr class="mt-2 border dark:border-neutral-700" />
	<div class="flex flex-row">
		<PluginManager actionList={() => actionList} profileSelector={() => profileSelector} />
		<SettingsView />
	</div>
</div>
