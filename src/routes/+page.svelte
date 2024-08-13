<script lang="ts">
	import type { DeviceInfo } from "$lib/DeviceInfo";
	import type { Profile } from "$lib/Profile";

	import { inspectedParentAction } from "$lib/propertyInspector";

	import ActionList from "../components/ActionList.svelte";
	import DeviceSelector from "../components/DeviceSelector.svelte";
	import DeviceView from "../components/DeviceView.svelte";
	import NoDevicesDetected from "../components/NoDevicesDetected.svelte";
	import ParentActionView from "../components/ParentActionView.svelte";
	import PluginManager from "../components/PluginManager.svelte";
	import ProfileSelector from "../components/ProfileSelector.svelte";
	import PropertyInspectorView from "../components/PropertyInspectorView.svelte";
	import SettingsView from "../components/SettingsView.svelte";

	let devices: { [id: string]: DeviceInfo } = {};
	let selectedDevice: string;
	let selectedProfiles: { [id: string]: Profile } = {};

	let actionList: ActionList;
	let deviceSelector: DeviceSelector;
	let profileSelector: ProfileSelector;
</script>

<div class="flex flex-col grow">
	{#if $inspectedParentAction}
		<ParentActionView bind:profile={selectedProfiles[selectedDevice]} />
	{/if}
	{#if Object.keys(devices).length > 0 && selectedProfiles}
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
	{#if !$inspectedParentAction}
		<DeviceSelector
			bind:devices={devices}
			bind:value={selectedDevice}
			bind:selectedProfiles={selectedProfiles}
			bind:this={deviceSelector}
			profileSelector={() => profileSelector}
		/>
		{#key selectedDevice}
			{#if selectedDevice && devices[selectedDevice]}
				<ProfileSelector
					bind:device={devices[selectedDevice]}
					bind:profile={selectedProfiles[selectedDevice]}
					bind:this={profileSelector}
				/>
			{/if}
		{/key}
	{/if}
	<ActionList bind:this={actionList} />
	<hr class="mt-2 border dark:border-neutral-700" />
	<div class="flex flex-row">
		<PluginManager actionList={() => actionList} deviceSelector={() => deviceSelector} />
		<SettingsView />
	</div>
</div>
