<script lang="ts">
    import Key from "./Key.svelte";
    import Slider from "./Slider.svelte";

	export let device: DeviceInfo;
	export let profile: Profile;
</script>

<div class="flex flex-row">
	{#each { length: device.sliders } as _, i}
		<Slider
			context="{device.id}.{profile.id}.Encoder.{i}.0"
			instance={profile.sliders[i]}
		/>
	{/each}

	<div class="flex flex-col">
		{#each { length: device.rows } as _, r}
			<div class="flex flex-row">
				{#each { length: device.columns } as _, c}
					<Key
						context="{device.id}.{profile.id}.Keypad.{(r * device.columns) + c}.0"
						instance={profile.keys[(r * device.columns) + c]}
					/>
				{/each}
			</div>
		{/each}
	</div>
</div>
