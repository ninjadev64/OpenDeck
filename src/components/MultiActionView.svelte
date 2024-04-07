<script lang="ts">
    import type { ActionInstance } from "$lib/ActionInstance";
    import type { Context } from "$lib/Context";
    import type { Profile } from "$lib/Profile";

	import { inspectedMultiAction } from "$lib/propertyInspector";
	import { invoke } from "@tauri-apps/api";

    import Key from "./Key.svelte";

	export let profile: Profile;

	let slot: ActionInstance[];
	$: {
		if ($inspectedMultiAction?.controller == "Encoder") {
			slot = profile.sliders[$inspectedMultiAction.position];
		} else {
			slot = profile.keys[$inspectedMultiAction!.position];
		}
	}

	function handleDragOver(event: DragEvent) {
		event.preventDefault();
		return true;
	}

	async function handleDrop({ dataTransfer }: DragEvent) {
		if (dataTransfer?.getData("action")) {
			let response: ActionInstance[] = await invoke("create_instance", { context: $inspectedMultiAction, action: JSON.parse(dataTransfer?.getData("action")) });
			if (response) slot = response;
		}
	}

	async function removeInstance(index: number) {
		await invoke("remove_instance", { context: slot[index].context });
		let temp = [...slot];
		temp.splice(index, 1);
		slot = temp;
	}

	let context: Context;
	context = null!;
</script>

<div class="px-6 pt-6 pb-4">
	<button class="float-right text-xl" on:click={() => $inspectedMultiAction = null}> ✕ </button>
	<h1 class="font-semibold text-2xl"> Multi Action </h1>
</div>

<div class="flex flex-col h-80 overflow-scroll">
	{#each slot as instance, index}
		<div class="flex flex-row items-center mx-4 my-1 bg-gray-100 rounded-md">
			<Key slot={[instance]} {context} active={false} scale={3/4} />
			<p class="ml-4 text-xl"> {instance.action.name} </p>
			<!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
			<img
				src="/rubbish.png"
				class="ml-auto mr-10 w-8 cursor-pointer"
				alt="Remove action"
				on:click={() => removeInstance(index)} on:keyup={() => removeInstance(index)}
			/>
		</div>
	{/each}
	<div
		class="flex flex-row items-center mx-4 mt-1 mb-4 bg-gray-100 border-2 border-dashed rounded-md"
		on:dragover={handleDragOver} on:drop={handleDrop}
		role="cell" tabindex="-1"
	>
		<img src="/cube.png" class="m-2 w-24 rounded-xl" alt="Add new action" />
		<p class="ml-4 text-xl text-gray-500"> Drop actions here </p>
	</div>
</div>
