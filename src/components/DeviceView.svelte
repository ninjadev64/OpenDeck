<script lang="ts">
    import Key from "./Key.svelte";
    import Slider from "./Slider.svelte";

	export let device: DeviceInfo;
</script>

<div class="flex flex-row">
	{#each { length: device.sliders } as _}
		<Slider />
	{/each}
	
	<div class="flex flex-col">
		{#each { length: device.rows } as _}
			<div class="flex flex-row">
				{#each { length: device.columns } as _}
					<Key />
				{/each}
			</div>
		{/each}
	</div>
</div>
