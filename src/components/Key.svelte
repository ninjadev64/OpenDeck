<script lang="ts">
	import type { ActionInstance } from "$lib/ActionInstance";
	import type { ActionState } from "$lib/ActionState";
	import type { Context } from "$lib/Context";

	import Clipboard from "phosphor-svelte/lib/Clipboard";
	import Copy from "phosphor-svelte/lib/Copy";
	import Pencil from "phosphor-svelte/lib/Pencil";
	import Trash from "phosphor-svelte/lib/Trash";
	import InstanceEditor from "./InstanceEditor.svelte";

	import { copiedContext, inspectedInstance, inspectedMultiAction, openContextMenu } from "$lib/propertyInspector";
	import { renderImage } from "$lib/rendererHelper";

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
	let pressed: boolean = false;

	let state: ActionState | undefined;
	$: {
		if (!slot || !slot.length) {
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
		} else if (payload.contents[0]?.context == slot?.[0]?.context) {
			slot[0] = payload.contents[0];
		}
	});

	listen("key_moved", ({ payload }: { payload: { context: Context, pressed: boolean }}) => {
		if (JSON.stringify(context) == JSON.stringify(payload.context)) {
			pressed = payload.pressed;
		};
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

	export let handlePaste: ((source: Context, destination: Context) => void) | undefined = undefined;
	async function paste() {
		if (!$copiedContext) return;
		if (handlePaste) handlePaste($copiedContext, context);
	}

	async function clear() {
		await invoke("clear_slot", { context });
		if (slot.map((instance) => instance.context).includes($inspectedInstance!)) inspectedInstance.set(null);
		showEditor = false;
		slot = [];
		inslot = slot;
	}

	let showAlert: boolean = false;
	let showOk: boolean = false;
	let timeouts: number[] = [];
	listen("show_alert", ({ payload }: { payload: string }) => {
		if (slot.length != 1 || payload != slot[0].context) return;
		timeouts.forEach(clearTimeout);
		showOk = false;
		showAlert = true;
		timeouts.push(setTimeout(() => showAlert = false, 1.5e3));
	});
	listen("show_ok", ({ payload }: { payload: string }) => {
		if (slot.length != 1 || payload != slot[0].context) return;
		timeouts.forEach(clearTimeout);
		showAlert = false;
		showOk = true;
		timeouts.push(setTimeout(() => showOk = false, 1.5e3));
	});

	let canvas: HTMLCanvasElement;
	export let size = 144;
	$: {
		if (!slot || slot.length == 0) {
			if (canvas) {
				let context = canvas.getContext("2d");
				if (context) context.clearRect(0, 0, canvas.width, canvas.height);
			}
		} else if (slot.length > 1) {
			renderImage(canvas, context, state!, null!, false, false, false, active, pressed);
		} else if (slot.length) {
			let instance = slot[0];
			let fallback = instance.action.states[instance.current_state].image ?? instance.action.icon;
			if (state) renderImage(canvas, context, state, fallback, showOk, showAlert, true, active, pressed);
		}
	}
</script>

<canvas
	bind:this={canvas}
	class="relative -m-2 border-2 dark:border-neutral-700 rounded-md select-none" class:-m-[2.06rem]={size == 192}
	width={size} height={size}
	style="scale: {(112 / size) * scale};"
	on:dragover on:drop
	draggable={slot && slot.length != 0} on:dragstart
	role="cell" tabindex="-1"
	on:click={select} on:keyup={select}
	on:contextmenu={contextMenu}
/>

{#if $openContextMenu && $openContextMenu?.context == context}
	<div
		class="absolute text-sm font-semibold w-32 dark:text-neutral-300 bg-neutral-100 dark:bg-neutral-700 border-2 dark:border-neutral-600 rounded-lg divide-y z-10"
		style="left: {$openContextMenu.x}px; top: {$openContextMenu.y}px;"
	>
		{#if !slot || slot.length == 0}
			<button
				class="flex flex-row p-2 w-full cursor-pointer items-center"
				on:click={paste}
			>
				<Clipboard size="18" color={document.documentElement.classList.contains("dark") ? "#DEDDDA" : "#77767B"} />
				<span class="ml-2"> Paste </span>
			</button>
		{:else}
			<button
				class="flex flex-row p-2 w-full cursor-pointer items-center"
				on:click={edit}
			>
				<Pencil size="18" color={document.documentElement.classList.contains("dark") ? "#DEDDDA" : "#77767B"} />
				<span class="ml-2"> Edit </span>
			</button>
			<button
				class="flex flex-row p-2 w-full cursor-pointer items-center"
				on:click={() => copiedContext.set(context)}
			>
				<Copy size="18" color={document.documentElement.classList.contains("dark") ? "#DEDDDA" : "#77767B"} />
				<span class="ml-2"> Copy </span>
			</button>
			<button
				class="flex flex-row p-2 w-full cursor-pointer items-center"
				on:click={clear}
			>
				<Trash size="18" color="#F66151" />
				<span class="ml-2"> Delete </span>
			</button>
		{/if}
	</div>
{/if}

{#if showEditor}
	<InstanceEditor bind:instance={slot[0]} bind:showEditor={showEditor} />
{/if}
