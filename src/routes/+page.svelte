<script lang="ts">
	import type { DeviceInfo } from "$lib/DeviceInfo";
	import type { Profile } from "$lib/Profile";

	import { inspectedMultiAction } from "$lib/propertyInspector";

	import ActionList from "../components/ActionList.svelte";
	import DeviceSelector from "../components/DeviceSelector.svelte";
	import DeviceView from "../components/DeviceView.svelte";
	import MultiActionView from "../components/MultiActionView.svelte";
    import PluginManager from "../components/PluginManager.svelte";
	import ProfileSelector from "../components/ProfileSelector.svelte";
	import PropertyInspectorView from "../components/PropertyInspectorView.svelte";

	let selectedDevice: DeviceInfo;
	$: _selectedDevice = selectedDevice;

	let selectedProfile: Profile;
	$: _selectedProfile = selectedProfile;
</script>

<div class="flex flex-col grow">
	{#if $inspectedMultiAction}
		<MultiActionView bind:profile={_selectedProfile} />
	{:else if selectedDevice && selectedProfile}
		<DeviceView bind:device={_selectedDevice} bind:profile={_selectedProfile} />
	{/if}

	{#if selectedProfile}
		<PropertyInspectorView bind:device={_selectedDevice} bind:profile={_selectedProfile} />
	{/if}
</div>

<div class="flex flex-col p-2 grow max-w-[18rem] h-full border-l">
	{#if !$inspectedMultiAction}
		<DeviceSelector bind:device={selectedDevice} />
		{#if selectedDevice}
			<ProfileSelector bind:device={_selectedDevice} bind:profile={selectedProfile} />
		{/if}
	{/if}
	<ActionList />
	<PluginManager />
</div>
