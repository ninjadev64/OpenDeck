<script lang="ts">
	import type { ActionInstance } from "$lib/ActionInstance";
	import type { ActionState } from "$lib/ActionState";
	import type { Context } from "$lib/Context";

	import Pencil from "phosphor-svelte/lib/Pencil";
	import Trash from "phosphor-svelte/lib/Trash";
	import InstanceEditor from "./InstanceEditor.svelte";

	import { inspectedInstance, inspectedMultiAction, openContextMenu } from "$lib/propertyInspector";
	import { getImage, renderImage } from "$lib/rendererHelper";

	import { invoke } from "@tauri-apps/api";
	import { listen } from "@tauri-apps/api/event";

	export let context: Context;

	// One-way binding for slot data.
	export let inslot: ActionInstance[];
	let slot: ActionInstance[];
	const update = (inslot: ActionInstance[]) => slot = inslot;
	$: update(inslot);

	export let active: boolean = true;
	export let scale: number = 1;

	let state: ActionState | undefined;
	$: {
		if (!slot.length) {
			state = undefined;
		} else if (slot.length > 1) {
			// @ts-expect-error
			state = {
				image: "/multi-action.png",
				name: "Multi Action",
				show: false
			};
		} else {
			state = slot[0].states[slot[0].current_state];
		}
	}

	listen("update_state", ({ payload }: { payload: { context: Context, contents: ActionInstance[] }}) => {
		if (JSON.stringify(payload.context) == JSON.stringify(context)) {
			slot = payload.contents;
		} else if (payload.contents[0]?.context == slot[0]?.context) {
			slot[0] = payload.contents[0];
		}
	});

	function select() {
		if (!slot || slot.length == 0) return;
		if (slot.length > 1) {
			inspectedMultiAction.set(context);
		} else {
			inspectedInstance.set(slot[0].context);
		}
	}

	async function contextMenu(event: MouseEvent) {
		if (!slot || slot.length == 0) return;
		event.preventDefault();
		if (!active) return;
		if (event.ctrlKey) return;
		$openContextMenu = { context, x: event.x, y: event.y };
	}

	let showEditor = false;
	function edit() {
		if (slot.length > 1) {
			inspectedMultiAction.set(context);
		} else {
			showEditor = true;
		}
	}

	async function clear() {
		await invoke("clear_slot", { context });
		if (slot.map((instance) => instance.context).includes($inspectedInstance!)) inspectedInstance.set(null);
		slot = [];
		inslot = slot;
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
		if (slot.length > 1) {
			image = state?.image!;
			if (active) renderImage(context, state!, null!, false, false, false);
		} else if (slot.length) {
			let instance = slot[0];
			let fallback = instance.action.states[instance.current_state].image ?? instance.action.icon;
			image = getImage(state?.image, fallback);
			if (active && state) renderImage(context, state, fallback, showOk > 0, showAlert > 0);
		}
	}
</script>

<div
	class="relative m-2 border-2 dark:border-neutral-700 rounded-md select-none"
	style="width: calc(8rem * {scale}); height: calc(8rem * {scale});"
	on:dragover on:drop
	draggable on:dragstart
	role="cell" tabindex="-1"
	on:click={select} on:keyup={select}
	on:contextmenu={contextMenu}
>
	{#if state}
		<img
			src={image}
			class="p-2 w-full rounded-xl"
			alt={slot.length == 1 ? slot[0].action.tooltip : "Multi Action"}
		/>
		{#if state.show}
			<div class="absolute flex justify-center w-full h-full top-0 left-0 pointer-events-none">
				<span
					style="
						font-size: calc({state.size}px * (112/72) * {scale});
						font-family: '{state.family}', sans-serif;
						color: {state.colour};
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

{#if $openContextMenu && $openContextMenu?.context == context}
	<div
		class="absolute text-sm font-semibold w-32 dark:text-neutral-300 bg-neutral-100 dark:bg-neutral-700 border-2 dark:border-neutral-600 rounded-lg divide-y z-10"
		style="left: {$openContextMenu.x}px; top: {$openContextMenu.y}px;"
	>
		<button
			class="flex flex-row p-2 w-full cursor-pointer items-center"
			on:click={edit}
		>
			<Pencil size="18" color={document.documentElement.classList.contains("dark") ? "#DEDDDA" : "#77767B"} />
			<span class="ml-2"> Edit </span>
		</button>
		<button
			class="flex flex-row p-2 w-full cursor-pointer items-center"
			on:click={clear}
		>
			<Trash size="18" color="#F66151" />
			<span class="ml-2"> Delete </span>
		</button>
	</div>
{/if}

{#if showEditor}
	<InstanceEditor bind:instance={slot[0]} bind:showEditor={showEditor} />
{/if}
