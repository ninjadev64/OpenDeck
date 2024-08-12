<script lang="ts">
    import type { ActionInstance } from "$lib/ActionInstance";
    import type { DeviceInfo } from "$lib/DeviceInfo";
    import type { Profile } from "$lib/Profile";

    import { inspectedInstance } from "$lib/propertyInspector";
    import { invoke } from "@tauri-apps/api";

	let iframes: { [context: string]: HTMLIFrameElement } = {};
	let iframeContainer: HTMLDivElement;
	let iframeClosePopup: HTMLButtonElement;
	let iframePopupsOpen: string[] = [];

	export let device: DeviceInfo;
	export let profile: Profile;

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
		if (iframe) {
			iframe.style.position = "";
			iframe.style.left = "";
			iframe.style.top = "";
			iframe.style.width = "100%";
			iframe.style.height = "100%";
			iframe.style.display = $inspectedInstance == context ? "block" : "none";
			iframe.contentWindow?.postMessage({ event: "windowClosed" }, "http://localhost:57118");
		}

		iframePopupsOpen = iframePopupsOpen.filter((e) => e != context);

		if (iframePopupsOpen.length == 0) {
			iframeContainer.style.position = "";
			iframeContainer.style.width = "";
			iframeContainer.style.height = "";
			iframeContainer.style.padding = "";
			iframeContainer.style.zIndex = "0";

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
			iframeContainer.style.zIndex = "20";

			iframeClosePopup.style.display = "block";
		} else if (data.event == "windowClosed") {
			closePopup(data.payload);
		} else if (data.event == "openUrl") {
			invoke("open_url", { url: data.payload });
		}
	});

	const nonNull = <T>(o: T | null): o is T => o != null;
	$: instances = profile.keys.filter(nonNull).concat(profile.sliders.filter(nonNull));
</script>

<div class="grow overflow-scroll bg-white dark:bg-neutral-900 border-t dark:border-neutral-700" bind:this={iframeContainer}>
	<button
		bind:this={iframeClosePopup}
		on:click={() => closePopup(iframePopupsOpen[iframePopupsOpen.length - 1])}
		class="absolute top-2 right-2 text-2xl dark:text-neutral-300 font-bold hidden"
	>
		✕
	</button>
	{#each instances as instance (instance.context)}
		{#if instance.action.property_inspector}
			<iframe
				title="Property inspector"
				class="w-full h-full hidden"
				class:!block={$inspectedInstance == instance.context}
				src={"http://localhost:57118/" + instance.action.property_inspector + "|opendeck_property_inspector"}
				name={instance.context}
				bind:this={iframes[instance.context]}
				on:load={() => iframeOnLoad(instance)}
			/>
		{/if}
	{/each}
</div>
