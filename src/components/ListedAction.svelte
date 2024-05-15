<script lang="ts">
	import type { Action } from "$lib/Action";

	export let action: Action;
	export let localisation: { name: string, tooltip: string };

	function handleDragStart(event: DragEvent) {
		event.dataTransfer?.setData("action", JSON.stringify(action));
	}
</script>

<div class="flex flex-row items-center mt-2 mb-2 space-x-2">
	<img
		src={"http://localhost:57118/" + action.icon}
		alt={localisation.tooltip}
		class="w-12 h-12 rounded-sm"
		draggable
		on:dragstart={handleDragStart}
	/>
	<span> {localisation.name} </span>
</div>
