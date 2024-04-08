<script lang="ts">
	import type { Action } from "$lib/Action";

	import ListedAction from "./ListedAction.svelte";

	import { invoke } from "@tauri-apps/api";

	let categories: { [name: string]: Action[] } = {};

	(async () => {
		categories = await invoke("get_categories");
	})();
</script>

<div class="grow mt-1 overflow-auto">
	{#each Object.entries(categories) as [ name, actions ]}
		<h3 class="text-xl font-semibold"> {name} </h3>
		{#each actions as action}
			{#if action.visible_in_action_list}
				<ListedAction {action} />
			{/if}
		{/each}
	{/each}
</div>
