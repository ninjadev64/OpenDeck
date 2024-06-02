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

	let selectedDevice: DeviceInfo;
	$: _selectedDevice = selectedDevice;

	let selectedProfile: Profile;
	$: _selectedProfile = selectedProfile;

	let actionList: ActionList;
	let profileSelector: ProfileSelector;
</script>

<div class="flex flex-col grow">
	{#if $inspectedMultiAction}
		<MultiActionView bind:profile={_selectedProfile} />
	{:else if selectedDevice && selectedProfile}
		<DeviceView bind:device={_selectedDevice} bind:profile={_selectedProfile} />
	{:else}
		<NoDevicesDetected />
	{/if}

	{#if selectedProfile}
		<PropertyInspectorView bind:device={_selectedDevice} bind:profile={_selectedProfile} />
	{/if}
</div>

<div class="flex flex-col p-2 grow max-w-[18rem] h-full border-l dark:border-neutral-700">
	{#if !$inspectedMultiAction}
		<DeviceSelector bind:device={selectedDevice} />
		{#if selectedDevice}
			<ProfileSelector
				bind:device={_selectedDevice}
				bind:profile={selectedProfile}
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
