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

	function handleDragOver(event: DragEvent) {
		event.preventDefault();
		return true;
	}

	async function handleDrop({ dataTransfer }: DragEvent, controller: string, position: number) {
		let context = `${profile.device}.${profile.id}.${controller}.${position}.0`;
		let array = controller == "Encoder" ? profile.sliders : profile.keys;
		if (dataTransfer?.getData("action")) {
			array[position] = await invoke("create_instance", { context, action: JSON.parse(dataTransfer?.getData("action")) });
			profile = profile;
		} else if (dataTransfer?.getData("controller")) {
			let oldArray = dataTransfer?.getData("controller") == "Encoder" ? profile.sliders : profile.keys;
			let oldPosition = parseInt(dataTransfer?.getData("position"));
			let response: ActionInstance = await invoke("move_instance", { context, instance: oldArray[oldPosition] });
			if (response) {
				array[position] = response;
				oldArray[oldPosition] = null;
				profile = profile;
			}
		}
	}

	function handleDragStart({ dataTransfer }: DragEvent, controller: string, position: number) {
		dataTransfer?.setData("controller", controller);
		dataTransfer?.setData("position", position.toString());
	}

	let iframes: { [context: string]: HTMLIFrameElement } = {};
	let iframeContainer: HTMLDivElement;
	let iframeClosePopup: HTMLButtonElement;
	let iframePopupsOpen: string[] = [];

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
		const info = JSON.stringify(await invoke("make_info", { plugin: instance.action.plugin }));

		iframe?.contentWindow?.postMessage({
			event: "connect",
			payload: [
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
			]
		}, "http://localhost:57118");
	}

	const closePopup = (context: string) => {
		const iframe = iframes[context];
		iframe.style.position = "";
		iframe.style.left = "";
		iframe.style.top = "";
		iframe.style.width = "100%";
		iframe.style.height = "100%";
		iframe.style.display = $inspectedInstance == context ? "block" : "none";
		iframe.contentWindow?.postMessage({ event: "windowClosed" }, "http://localhost:57118");

		iframePopupsOpen = iframePopupsOpen.filter((e) => e != context);

		if (iframePopupsOpen.length == 0) {
			iframeContainer.style.position = "";
			iframeContainer.style.width = "";
			iframeContainer.style.height = "";
			iframeContainer.style.padding = "";

			iframeClosePopup.style.display = "none";
		}
	}

	window.addEventListener("message", ({ data }) => {
		if (data.event == "windowOpened") {
			const iframe = iframes[data.payload];
			iframe.style.position = "absolute";
			iframe.style.left = "36px";
			iframe.style.top = "36px";
			iframe.style.width = "calc(100% - 72px)";
			iframe.style.height = "calc(100% - 72px)";
			iframe.style.display = "block";

			iframePopupsOpen.push(data.payload);

			iframeContainer.style.position = "absolute";
			iframeContainer.style.width = "100%";
			iframeContainer.style.height = "100%";
			iframeContainer.style.padding = "36px";

			iframeClosePopup.style.display = "block";
		} else if (data.event == "windowClosed") {
			closePopup(data.payload);
		}
	});

	const nonNull = <T>(o: T | null): o is T => o != null;
	$: nonNullInstances = profile.keys.filter(nonNull).concat(profile.sliders.filter(nonNull));
</script>

<div class="flex flex-row">
	{#each { length: device.sliders } as _, i}
		<Slider
			context="{device.id}.{profile.id}.Encoder.{i}.0"
			bind:instance={profile.sliders[i]}
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
						context="{device.id}.{profile.id}.Keypad.{(r * device.columns) + c}.0"
						bind:instance={profile.keys[(r * device.columns) + c]}
						on:dragover={handleDragOver}
						on:drop={(event) => handleDrop(event, "Keypad", (r * device.columns) + c)}
						on:dragstart={(event) => handleDragStart(event, "Keypad", (r * device.columns) + c)}
					/>
				{/each}
			</div>
		{/each}
	</div>
</div>

<div class="grow overflow-scroll bg-white border-t" bind:this={iframeContainer}>
	<button
		bind:this={iframeClosePopup}
		on:click={() => closePopup(iframePopupsOpen[iframePopupsOpen.length - 1])}
		class="absolute top-2 right-2 text-2xl font-bold hidden"
	>
		✕
	</button>
	{#each nonNullInstances as instance (instance.context)}
		{#if instance.action.property_inspector}
			<iframe
				title="Property inspector"
				class="w-full h-full hidden"
				class:!block={$inspectedInstance == instance.context}
				src={"http://localhost:57118" + instance.action.property_inspector + "|opendeck_property_inspector"}
				name={instance.context}
				bind:this={iframes[instance.context]}
				on:load={() => iframeOnLoad(instance)}
			/>
		{/if}
	{/each}
</div>
