<script lang="ts">
	import type { ActionInstance } from "$lib/ActionInstance";
	import type { Context } from "$lib/Context";
	import type { DeviceInfo } from "$lib/DeviceInfo";
	import type { Profile } from "$lib/Profile";

	import Key from "./Key.svelte";
	import Slider from "./Slider.svelte";

	import { invoke } from "@tauri-apps/api";

	export let device: DeviceInfo;
	export let profile: Profile;

	export let selectedDevice: string;

	function handleDragOver(event: DragEvent) {
		event.preventDefault();
		return true;
	}

	async function handleDrop({ dataTransfer }: DragEvent, controller: string, position: number) {
		let context = { device: device.id, profile: profile.id, controller, position };
		let array = controller == "Encoder" ? profile.sliders : profile.keys;
		if (dataTransfer?.getData("action")) {
			let action = JSON.parse(dataTransfer?.getData("action"));
			if (array[position].length >= 1 && (!array[position][0].action.supported_in_multi_actions || !action.supported_in_multi_actions)) {
				return;
			}
			array[position] = await invoke("create_instance", { context, action });
			profile = profile;
		} else if (dataTransfer?.getData("controller")) {
			let oldArray = dataTransfer?.getData("controller") == "Encoder" ? profile.sliders : profile.keys;
			let oldPosition = parseInt(dataTransfer?.getData("position"));
			let response: ActionInstance[] = await invoke("move_slot", {
				source: { device: device.id, profile: profile.id, controller: dataTransfer?.getData("controller"), position: oldPosition },
				destination: context,
				retain: false
			});
			if (response) {
				array[position] = response;
				oldArray[oldPosition] = [];
				profile = profile;
			}
		}
	}

	function handleDragStart({ dataTransfer }: DragEvent, controller: string, position: number) {
		dataTransfer?.setData("controller", controller);
		dataTransfer?.setData("position", position.toString());
	}

	async function handlePaste(source: Context, destination: Context) {
		let response: ActionInstance[] = await invoke("move_slot", { source, destination, retain: true });
		if (response) {
			profile.keys[destination.position] = response;
			profile = profile;
		}
	}
</script>

{#key device}
	<div class="flex flex-row" class:hidden={selectedDevice != device.id}>
		{#each { length: device.sliders } as _, i}
			<Slider
				context={{ device: device.id, profile: profile.id, controller: "Encoder", position: i }}
				bind:slot={profile.sliders[i]}
				on:dragover={handleDragOver}
				on:drop={(event) => handleDrop(event, "Encoder", i)}
				on:dragstart={(event) => handleDragStart(event, "Encoder", i)}
			/>
		{/each}

		<div class="flex flex-col">
			{#each { length: device.rows } as _, r}
				<div class="flex flex-row">
					{#each { length: device.columns } as _, c}
						<Key
							context={{ device: device.id, profile: profile.id, controller: "Keypad", position: (r * device.columns) + c }}
							bind:inslot={profile.keys[(r * device.columns) + c]}
							on:dragover={handleDragOver}
							on:drop={(event) => handleDrop(event, "Keypad", (r * device.columns) + c)}
							on:dragstart={(event) => handleDragStart(event, "Keypad", (r * device.columns) + c)}
							{handlePaste}
							size={device.id.startsWith("sd-") && device.rows == 4 && device.columns == 8 ? 192 : 144}
						/>
					{/each}
				</div>
			{/each}
		</div>
	</div>
{/key}
