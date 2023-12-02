<script lang="ts">
	import ActionList from "../components/ActionList.svelte";
	import DeviceSelector from "../components/DeviceSelector.svelte";
	import DeviceView from "../components/DeviceView.svelte";

	let selectedDevice: DeviceInfo;
</script>

<div class="grow">
	{#if selectedDevice}
		<DeviceView device={selectedDevice} />
	{/if}
</div>

<div class="flex flex-col p-2 grow max-w-[18rem] h-full border-l">
	<DeviceSelector on:select={({ detail }) => selectedDevice = detail} />
	<ActionList />
</div>
