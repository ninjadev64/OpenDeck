<script lang="ts">
	import type { ActionInstance } from "$lib/ActionInstance";
	import type { Context } from "$lib/Context";

	import { inspectedInstance } from "$lib/propertyInspector";
	import { getImage } from "$lib/rendererHelper";

	import { invoke } from "@tauri-apps/api/core";
	import { listen } from "@tauri-apps/api/event";

	export let context: Context;
	export let slot: ActionInstance | null;

	$: state = slot ? slot.states[slot.current_state] : undefined;

	listen("update_state", ({ payload }: { payload: { context: string; contents: ActionInstance | null } }) => {
		if (payload.context == slot?.context) slot = payload.contents;
	});

	function select() {
		if (slot) inspectedInstance.set(`${context.device}.${context.profile}.${context.controller}.${context.position}.0`);
	}

	async function clear(event: MouseEvent) {
		if (!slot) return;
		event.preventDefault();
		if (event.ctrlKey) return;
		await invoke("remove_instance", { context: slot.context });
		if ($inspectedInstance == slot.context) inspectedInstance.set(null);
		slot = null;
	}

	let showAlert = 0;
	let showOk = 0;
	let timeouts: number[] = [];
	listen("show_alert", ({ payload }: { payload: string }) => {
		if (!slot || payload != slot.context) return;
		timeouts.forEach(clearTimeout);
		showOk = 0;
		showAlert = 1;
		timeouts.push(setTimeout(() => showAlert = 2, 1e3));
		timeouts.push(setTimeout(() => showAlert = 0, 2e3));
	});
	listen("show_ok", ({ payload }: { payload: string }) => {
		if (!slot || payload != slot.context) return;
		timeouts.forEach(clearTimeout);
		showAlert = 0;
		showOk = 1;
		timeouts.push(setTimeout(() => showOk = 2, 1e3));
		timeouts.push(setTimeout(() => showOk = 0, 2e3));
	});

	let image: string;
	$: {
		if (slot) {
			image = getImage(state?.image, slot.action.states[slot.current_state].image ?? slot.action.icon);
		}
	}
</script>

<!-- svelte-ignore a11y-no-static-element-interactions -->
<div
	class="relative flex items-center m-2 w-20 h-144 border-2 dark:border-neutral-700 rounded-md select-none"
	draggable
	on:dragstart
	on:dragover
	on:drop
	on:click|stopPropagation={select}
	on:keyup|stopPropagation={select}
	on:contextmenu={clear}
>
	{#if state}
		<img
			src={image}
			class="p-2 w-full rounded-xl"
			alt={slot ? slot.action.tooltip : ""}
		/>
		{#if state.show}
			<div class="absolute flex justify-center w-full aspect-square top-[50%] -translate-y-1/2 left-0 pointer-events-none">
				<span
					style={`
						font-size: ${state.size}px;
						color: ${state.colour};
						scale: 0.5;
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
				class="absolute left-0 p-1 w-full aspect-square top-[50%] -translate-y-1/2 transition-opacity duration-1000"
				class:opacity-0={showAlert == 2}
			/>
		{/if}
		{#if showOk}
			<img
				src="/ok.png"
				alt="OK"
				class="absolute left-0 p-1 w-full aspect-square top-[50%] -translate-y-1/2 transition-opacity duration-1000"
				class:opacity-0={showOk == 2}
			/>
		{/if}
	{/if}
</div>
