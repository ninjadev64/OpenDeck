<script lang="ts">
	import ActionList from "../components/ActionList.svelte";
	import DeviceSelector from "../components/DeviceSelector.svelte";
	import DeviceView from "../components/DeviceView.svelte";
	import ProfileSelector from "../components/ProfileSelector.svelte";

	let selectedDevice: DeviceInfo;
	$: _selectedDevice = selectedDevice;

	let selectedProfile: Profile;
	$: _selectedProfile = selectedProfile;
</script>

<div class="grow">
	{#if selectedDevice && selectedProfile}
		<DeviceView bind:device={_selectedDevice} bind:profile={_selectedProfile} />
	{/if}
</div>

<div class="flex flex-col p-2 grow max-w-[18rem] h-full border-l">
	<DeviceSelector bind:device={selectedDevice} />
	{#if selectedDevice}
		<ProfileSelector bind:device={_selectedDevice} bind:profile={selectedProfile} />
	{/if}
	<ActionList />
</div>
