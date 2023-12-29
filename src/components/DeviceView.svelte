<script lang="ts">
	import type { ActionInstance } from "$lib/ActionInstance";
	import type { DeviceInfo } from "$lib/DeviceInfo";
	import type { Profile } from "$lib/Profile";

    import { inspectedInstance } from "$lib/propertyInspector";

	import Key from "./Key.svelte";
	import Slider from "./Slider.svelte";

    import { invoke } from "@tauri-apps/api";

	export let device: DeviceInfo;
	export let profile: Profile;

	let iframes: { [context: string]: HTMLIFrameElement } = {};
	async function iframeOnLoad(instance: ActionInstance) {
		const iframe = iframes[instance.context];
		const split = instance.context.split(".");

		let coordinates: { row: number, column: number };
		if (split[2] == "Encoder") {
			coordinates = { row: 0, column: parseInt(split[3]) };
		} else {
			coordinates = { row: Math.floor(parseInt(split[3]) / device.rows), column: parseInt(split[3]) % device.columns };
		}

		if (instance == null || !iframe.src || !iframe.src.startsWith("http://localhost:57118")) return;
		const info = await invoke("make_info", { plugin: instance.action.plugin });

		iframe?.contentWindow?.postMessage([
			57116,
			instance.context,
			"registerPropertyInspector",
			info,
			JSON.stringify({
				action: instance.action.uuid,
				context: instance.context,
				device: split[0],
				payload: {
					settings: instance.settings,
					coordinates
				}
			})
		], "http://localhost:57118");
	}

	const nonNull = <T>(o: T | null): o is T => o != null;
	$: nonNullInstances = profile.keys.filter(nonNull).concat(profile.sliders.filter(nonNull));
</script>

<div class="flex flex-row">
	{#each { length: device.sliders } as _, i}
		<Slider
			context="{device.id}.{profile.id}.Encoder.{i}.0"
			bind:instance={profile.sliders[i]}
		/>
	{/each}

	<div class="flex flex-col">
		{#each { length: device.rows } as _, r}
			<div class="flex flex-row">
				{#each { length: device.columns } as _, c}
					<Key
						context="{device.id}.{profile.id}.Keypad.{(r * device.columns) + c}.0"
						bind:instance={profile.keys[(r * device.columns) + c]}
					/>
				{/each}
			</div>
		{/each}
	</div>
</div>

<div class="grow overflow-scroll border-t">
	{#each nonNullInstances as instance (instance.context)}
		{#if instance.action.property_inspector}
			<iframe
				title="Property inspector"
				class="w-full h-full hidden"
				class:!block={$inspectedInstance == instance.context}
				src={"http://localhost:57118" + instance.action.property_inspector + "|opendeck_property_inspector"}
				bind:this={iframes[instance.context]}
				on:load={() => iframeOnLoad(instance)}
			/>
		{/if}
	{/each}
</div>
