<script lang="ts">
	import type { Action } from "$lib/Action";

	import ListedAction from "./ListedAction.svelte";

	import { localisations } from "$lib/settings";
	import { invoke } from "@tauri-apps/api";

	let categories: { [name: string]: Action[] } = {};
	export async function reload() {
		categories = await invoke("get_categories");
	}
	reload();

	function localiseAction(action: Action, localisations: { [plugin: string]: any } | null): { name: string, tooltip: string } {
		let { name, tooltip } = { name: action.name, tooltip: action.tooltip };
		if (localisations && localisations[action.plugin] && localisations[action.plugin][action.uuid]) {
			let localised = localisations[action.plugin][action.uuid];
			if (localised.Name) name = localised.Name;
			if (localised.Tooltip) tooltip = localised.Tooltip;
		}
		return { name, tooltip };
	}
</script>

<div class="grow mt-1 overflow-auto">
	{#each Object.entries(categories) as [ name, actions ]}
		<details open class="mb-2">
			<summary class="text-xl font-semibold dark:text-neutral-300"> {name} </summary>
			{#each actions as action}
				{#if action.visible_in_action_list}
					<ListedAction {action} localisation={localiseAction(action, $localisations)} />
				{/if}
			{/each}
		</details>
	{/each}
</div>
