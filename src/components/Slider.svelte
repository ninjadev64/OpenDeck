<script lang="ts">
	import type { ActionInstance } from "$lib/ActionInstance";

	import { inspectedInstance } from "$lib/propertyInspector";

	import { invoke } from "@tauri-apps/api";
	import { convertFileSrc } from "@tauri-apps/api/tauri";

	export let context: string;
	export let instance: ActionInstance | null;

	$: state = instance?.states[instance?.current_state];

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
</script>

<div
	class="relative flex items-center m-2 w-20 h-144 border-2 rounded-md select-none"
	on:dragover={handleDragOver}
	on:drop={handleDrop}
	role="cell" tabindex="-1"
>
	{#if instance && state}
		<!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
		<img
			src={convertFileSrc(state.image)}
			class="p-2 w-full rounded-xl"
			alt={instance.action.tooltip}
			on:click={clear} on:keyup={clear}
			on:contextmenu={(event) => {
				event.preventDefault();
				inspectedInstance.set(context);
			}}
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
	{/if}
</div>
