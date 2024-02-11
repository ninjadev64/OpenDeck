<script lang="ts">
	import type { ActionInstance } from "$lib/ActionInstance";

	import { inspectedInstance } from "$lib/propertyInspector";

	import { invoke } from "@tauri-apps/api";
	import { listen } from "@tauri-apps/api/event";
	import { convertFileSrc } from "@tauri-apps/api/tauri";

	export let context: string;
	export let instance: ActionInstance | null;

	$: state = instance?.states[instance?.current_state];
	let oldImage: string;

	listen("update_state", ({ payload }: { payload: string }) => {
		let i = JSON.parse(payload);
		if (i.context == context) instance = i;
	});

	function handleDragOver(event: DragEvent) {
		event.preventDefault();
		return true;
	}

	async function handleDrop({ dataTransfer }: DragEvent) {
		let action = dataTransfer?.getData("action");
		if (!action) return;
		instance = JSON.parse(await invoke("create_instance", { context, action: JSON.parse(action) }));
	}

	async function clear(event: MouseEvent | KeyboardEvent) {
		if (event.ctrlKey) return;
		instance = JSON.parse(await invoke("clear_slot", { context }));
		if ($inspectedInstance == context) inspectedInstance.set(null);
	}

	let showAlert = 0;
	let showOk = 0;
	let timeouts: number[] = [];
	listen("show_alert", ({ payload }: { payload: string }) => {
		if (payload != context) return;
		timeouts.forEach(clearTimeout);
		showOk = 0;
		showAlert = 1;
		timeouts.push(setTimeout(() => showAlert = 2, 1e3));
		timeouts.push(setTimeout(() => showAlert = 0, 2e3));
	});
	listen("show_ok", ({ payload }: { payload: string }) => {
		if (payload != context) return;
		timeouts.forEach(clearTimeout);
		showAlert = 0;
		showOk = 1;
		timeouts.push(setTimeout(() => showOk = 2, 1e3));
		timeouts.push(setTimeout(() => showOk = 0, 2e3));
	});

	function getImage(image: string): string {
		if (!image.startsWith("data:")) return convertFileSrc(image);
		const svgxmlre = /^data:image\/svg\+xml,(.+)/;
		const base64re = /^data:image\/(apng|avif|gif|jpeg|png|svg\+xml|webp|bmp|x-icon|tiff);base64,([A-Za-z0-9+/]+={0,2})?/;
		if (svgxmlre.test(image)) {
			image = "data:image/svg+xml;base64," + btoa(decodeURIComponent((svgxmlre.exec(image) as RegExpExecArray)[1].replace(/\;$/, "")));
		}
		if (base64re.test(image)) {
			let exec = base64re.exec(image)!;
			if (!exec[2]) return oldImage;
			else image = exec[0];
		}
		oldImage = image;
		return image;
	}
</script>

<div
	class="relative m-2 w-32 h-32 border-2 rounded-md select-none"
	on:dragover={handleDragOver}
	on:drop={handleDrop}
	role="cell" tabindex="-1"
>
	{#if instance && state}
		<!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
		<img
			src={getImage(state.image)}
			class="p-2 w-full rounded-xl"
			alt={instance.action.tooltip}
			on:click={clear} on:keyup={clear}
			on:contextmenu={(event) => {
				event.preventDefault();
				inspectedInstance.set(context);
			}}
		/>
		{#if state.show}
			<div class="absolute flex justify-center w-full h-full top-0 left-0 pointer-events-none">
				<span
					style={`
						font-size: ${state.size}px;
						color: ${state.colour};
					`}
					class:self-start={state.alignment == "top"}
					class:self-center={state.alignment == "middle"}
					class:self-end={state.alignment == "bottom"}
					class:font-bold={state.style.includes("Bold")}
					class:italic={state.style.includes("Italic")}
					class:underline={state.underline}
				>
					{state.text}
				</span>
			</div>
		{/if}
		{#if showAlert > 0}
			<img
				src="/alert.png"
				alt="Alert"
				class="absolute top-0 left-0 p-2 w-full h-full transition-opacity duration-1000"
				class:opacity-0={showAlert == 2}
			/>
		{/if}
		{#if showOk}
			<img
				src="/ok.png"
				alt="OK"
				class="absolute top-0 left-0 p-2 w-full h-full transition-opacity duration-1000"
				class:opacity-0={showOk == 2}
			/>
		{/if}
	{/if}
</div>
