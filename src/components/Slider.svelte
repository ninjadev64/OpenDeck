<script lang="ts">
    import { invoke } from "@tauri-apps/api";
	import { convertFileSrc } from "@tauri-apps/api/tauri";

	export let context: ActionContext;
	export let instance: ActionInstance | null;

	function handleDragOver(event: DragEvent) {
		event.preventDefault();
		return true;
	}

	async function handleDrop({ dataTransfer }: DragEvent) {
		let action = dataTransfer?.getData("action");
		if (!action) return;
		instance = JSON.parse(await invoke("create_instance", { context, action: JSON.parse(action) }));
	}
</script>

<div
	class="flex items-center m-2 w-20 h-144 border-2 rounded-md"
	on:dragover={handleDragOver}
	on:drop={handleDrop}
	role="cell" tabindex="-1"
>
	{#if instance}
		<img
			src={convertFileSrc(instance.states[instance.current_state].image)}
			class="p-2 w-full rounded-xl"
			alt={instance.action.tooltip}
		/>
	{/if}
</div>
