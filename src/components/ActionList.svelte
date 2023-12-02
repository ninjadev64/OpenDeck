<script lang="ts">
	import { invoke } from "@tauri-apps/api";
    import ListedAction from "./ListedAction.svelte";

	let categories: { [name: string]: Action[] } = {};

	invoke("get_categories").then((data) => {
		categories = JSON.parse(data as string)
	});
</script>

<div class="mt-1 overflow-auto">
	{#each Object.entries(categories) as [ name, actions ]}
		<h3 class="text-xl font-semibold"> {name} </h3>
		{#each actions as action}
			{#if action.visible_in_action_list}
				<ListedAction {action} />
			{/if}
		{/each}
	{/each}
</div>
