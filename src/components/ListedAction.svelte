<script lang="ts">
	import { convertFileSrc } from "@tauri-apps/api/tauri";
	export let action: Action;

	function handleDragStart(event: DragEvent) {
		event.dataTransfer?.setData("action", JSON.stringify(action));
	}
</script>

<div class="flex flex-row items-center mt-2 mb-2 space-x-2">
	<img
		src={convertFileSrc(action.icon)}
		alt={action.tooltip}
		class="w-12 h-12 rounded-sm"
		draggable
		on:dragstart={handleDragStart}
	/>
	<span> {action.name} </span>
</div>
