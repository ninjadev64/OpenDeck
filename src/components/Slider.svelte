<script lang="ts">
	import type { ActionInstance } from "$lib/ActionInstance";
	import type { Context } from "$lib/Context";

	import { inspectedInstance } from "$lib/propertyInspector";
	import { getImage } from "$lib/rendererHelper";

	import { invoke } from "@tauri-apps/api";
	import { listen } from "@tauri-apps/api/event";

	export let context: Context;
	export let slot: ActionInstance[];

	$: state = slot[0]?.states[slot[0]?.current_state];

	listen("update_state", ({ payload }: { payload: { context: Context, contents: ActionInstance[] }}) => {
		if (JSON.stringify(payload.context) == JSON.stringify(context)) slot = payload.contents;
	});

	function select() {
		inspectedInstance.set(`${context.device}.${context.profile}.${context.controller}.${context.position}.0`);
	}

	async function clear(event: MouseEvent) {
		event.preventDefault();
		if (event.ctrlKey) return;
		await invoke("clear_slot", { context });
		if ($inspectedInstance == slot[0]?.context) inspectedInstance.set(null);
		slot = [];
	}

	let showAlert = 0;
	let showOk = 0;
	let timeouts: number[] = [];
	listen("show_alert", ({ payload }: { payload: string }) => {
		if (slot.length != 1 || payload != slot[0].context) return;
		timeouts.forEach(clearTimeout);
		showOk = 0;
		showAlert = 1;
		timeouts.push(setTimeout(() => showAlert = 2, 1e3));
		timeouts.push(setTimeout(() => showAlert = 0, 2e3));
	});
	listen("show_ok", ({ payload }: { payload: string }) => {
		if (slot.length != 1 || payload != slot[0].context) return;
		timeouts.forEach(clearTimeout);
		showAlert = 0;
		showOk = 1;
		timeouts.push(setTimeout(() => showOk = 2, 1e3));
		timeouts.push(setTimeout(() => showOk = 0, 2e3));
	});

	let image: string;
	$: {
		if (slot.length) {
			image = getImage(state?.image, slot[0].action.states[slot[0].current_state].image ?? slot[0].action.icon);
		}
	}
</script>

<div
	class="relative flex items-center m-2 w-20 h-144 border-2 rounded-md select-none"
	on:dragover on:drop
	draggable on:dragstart
	role="cell" tabindex="-1"
>
	{#if state}
		<!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
		<img
			src={image}
			class="p-2 w-full rounded-xl"
			alt={slot.length == 1 ? slot[0].action.tooltip : ""}
			on:click={select} on:keyup={select}
			on:contextmenu={clear}
		/>
		{#if state.show}
			<div class="absolute flex justify-center w-full aspect-square top-[50%] -translate-y-1/2 left-0 pointer-events-none">
				<span
					style="
						font-size: {state.size}px;
						color: {state.colour};
						scale: 0.5;
					"
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
				class="absolute top-0 left-0 p-1 w-full h-full transition-opacity duration-1000"
				class:opacity-0={showAlert == 2}
			/>
		{/if}
		{#if showOk}
			<img
				src="/ok.png"
				alt="OK"
				class="absolute top-0 left-0 p-1 w-full h-full transition-opacity duration-1000"
				class:opacity-0={showOk == 2}
			/>
		{/if}
	{/if}
</div>
