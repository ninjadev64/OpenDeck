<script lang="ts">
	import Key from "./Key.svelte";
	import Slider from "./Slider.svelte";

    import { inspectedInstance } from "$lib/propertyInspector";

	export let device: DeviceInfo;
	export let profile: Profile;

	let iframes: { [context: string]: HTMLIFrameElement } = {};
</script>

<div class="flex flex-row">
	{#each { length: device.sliders } as _, i}
		<Slider
			context="{device.id}.{profile.id}.Encoder.{i}.0"
			bind:instance={profile.sliders[i]}
			bind:iframe={iframes[`${device.id}.${profile.id}.Encoder.${i}.0`]}
		/>
	{/each}

	<div class="flex flex-col">
		{#each { length: device.rows } as _, r}
			<div class="flex flex-row">
				{#each { length: device.columns } as _, c}
					<Key
						context="{device.id}.{profile.id}.Keypad.{(r * device.columns) + c}.0"
						bind:instance={profile.keys[(r * device.columns) + c]}
						bind:iframe={iframes[`${device.id}.${profile.id}.Keypad.${(r * device.columns) + c}.0`]}
					/>
				{/each}
			</div>
		{/each}
	</div>
</div>

<div class="grow overflow-scroll border-t">
	{#each { length: device.sliders } as _, i}
		<iframe
			title="Property inspector"
			class="w-full h-full hidden"
			class:!block={$inspectedInstance == `${device.id}.${profile.id}.Encoder.${i}.0`}
			bind:this={iframes[`${device.id}.${profile.id}.Encoder.${i}.0`]}
		/>
	{/each}
	{#each { length: device.rows * device.columns } as _, i}
		<iframe
			title="Property inspector"
			class="w-full h-full hidden"
			class:!block={$inspectedInstance == `${device.id}.${profile.id}.Keypad.${i}.0`}
			bind:this={iframes[`${device.id}.${profile.id}.Keypad.${i}.0`]}
		/>
	{/each}
</div>
