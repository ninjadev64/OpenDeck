<script lang="ts">
	import type { ActionInstance } from "$lib/ActionInstance";

	import { getImage } from "$lib/rendererHelper";

	import { invoke } from "@tauri-apps/api";

	export let instance: ActionInstance;
	export let showEditor: boolean;

	let state: number = 0;
	let bold: boolean;
	let italic: boolean;

	let fileInput: HTMLInputElement;

	function update(instance: ActionInstance) {
		bold = instance.states[state].style.includes("Bold");
		italic = instance.states[state].style.includes("Italic");
	}
	$: update(instance);
	$: invoke("set_state", { instance, state });
</script>

<div class="absolute left-1/2 top-1/2 -translate-x-1/2 -translate-y-1/2 p-2 dark:text-neutral-300 bg-neutral-100 dark:bg-neutral-700 border-2 dark:border-neutral-600 rounded-lg z-10">
	<div class="flex flex-row">
		<div class="select-wrapper ml-2 mt-2 mb-1 w-full">
			<select class="w-full" bind:value={state}>
				{#each instance.states as _, i}
					<option value={i}> State {i + 1} </option>
				{/each}
			</select>
		</div>
		<button class="ml-3 mr-2 float-right text-xl dark:text-neutral-300" on:click={() => showEditor = false}> ✕ </button>
	</div>
	<div class="flex flex-row">
		<!-- svelte-ignore a11y-no-noninteractive-element-interactions -->
		<img
			src={getImage(instance.states[state].image, instance.action.states[state].image ?? instance.action.icon)}
			class="mx-1 my-auto p-1 w-32 h-min aspect-square rounded-xl cursor-pointer"
			alt="State {state}"
			on:click={() => fileInput.click()} on:keyup={() => fileInput.click()}
			on:contextmenu={(event) => {
				event.preventDefault();
				if (event.ctrlKey) {
					instance.states[state].image = "data:image/png;base64,iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVQIW2NgYGD4DwABBAEAwS2OUAAAAABJRU5ErkJggg==";
				} else {
					instance.states[state].image = instance.action.states[state].image;
				}
			}}
		/>
		<input
			bind:this={fileInput}
			type="file" class="hidden" accept="image/*"
			on:change={async () => {
				if (!fileInput.files || fileInput.files.length == 0) return;
				const reader = new FileReader();
				
				reader.onload = async function() {

					var canvas = document.createElement("canvas");
					canvas.width = 144;
					canvas.height = 144;
					let context = canvas.getContext("2d");
					if (!context) return;

					context.imageSmoothingQuality = "high";

					let image = document.createElement("img");
					image.crossOrigin = "anonymous";

					image.src = getImage(reader.result?.toString(), reader.result?.toString())

					await new Promise((resolve) => {
						image.onload = resolve;
					});
					
					var xoffset = 0;
					var yoffset = 0;
					var xscaled = canvas.width;
					var yscaled = canvas.height;

					if (image.width > image.height) {
						var ratio = image.height/image.width;
						yoffset =canvas.height*ratio*.5
						yscaled =canvas.height*ratio;

					} else if (image.width < image.height) {
						var ratio = image.width/image.height;
						xoffset =canvas.width*ratio*.5
						xscaled =canvas.width*ratio;
					}

					context.clearRect(0, 0, canvas.width, canvas.height);
					
					context.drawImage(image, xoffset, yoffset, xscaled, yscaled);

					instance.states[state].image = canvas.toDataURL();
				
				}
				
				reader.readAsDataURL(fileInput.files[0]);
				
			}}
		/>

		<div class="flex flex-col pl-1 pr-2 py-2 space-y-2">
			<div class="flex flex-row space-x-2">
				<span> Text </span>
				<textarea
					bind:value={instance.states[state].text}
					disabled={!instance.action.user_title_enabled}
					rows=1
					class="w-full px-1 dark:text-neutral-300 dark:bg-neutral-600 rounded-md outline-none resize-none"
				/>
			</div>
			<div class="flex flex-row items-center">
				<span class="mr-2"> Colour </span>
				<input
					type="color"
					bind:value={instance.states[state].colour}
					disabled={!instance.action.user_title_enabled}
					class="mr-2 px-0.5 dark:bg-neutral-600 rounded-md outline-none"
				/>
				<span class="mr-2"> Show </span>
				<input
					type="checkbox"
					bind:checked={instance.states[state].show}
					disabled={!instance.action.user_title_enabled}
					class="mr-4 mt-1 scale-125"
				/>
				<select
					bind:value={instance.states[state].alignment}
					class="!px-1 !py-0.5"
				>
					<option value="top"> Top </option>
					<option value="middle"> Middle </option>
					<option value="bottom"> Bottom </option>
				</select>
			</div>
			<div>
				<input
					list="families"
					bind:value={instance.states[state].family}
					placeholder="Font family"
					class="w-full px-1 dark:text-neutral-300 dark:bg-neutral-600 rounded-md outline-none"
				>
				<datalist id="families">
					<option value="Liberation Sans"> Liberation Sans </option>
					<option value="Archivo Black"> Archivo Black </option>
					<option value="Comic Neue"> Comic Neue </option>
					<option value="Courier Prime"> Courier Prime </option>
					<option value="Tinos"> Tinos </option>
					<option value="Anton"> Anton </option>
					<option value="Liberation Serif"> Liberation Serif </option>
					<option value="Open Sans"> Open Sans </option>
					<option value="Fira Sans"> Fira Sans </option>
				</datalist>
			</div>
			<div class="flex flex-row">
				<span class="mr-3 font-bold"> B </span>
				<input
					type="checkbox"
					bind:checked={bold}
					on:change={() => instance.states[state].style = (bold && italic ? "Bold Italic" : bold ? "Bold" : italic ? "Italic" : "Regular")}
					disabled={!instance.action.user_title_enabled}
					class="mr-4 mt-1 scale-125"
				/>
				<span class="mr-3 italic"> I </span>
				<input
					type="checkbox"
					bind:checked={italic}
					on:change={() => instance.states[state].style = (bold && italic ? "Bold Italic" : bold ? "Bold" : italic ? "Italic" : "Regular")}
					disabled={!instance.action.user_title_enabled}
					class="mr-4 mt-1 scale-125"
				/>
				<span class="mr-3 underline"> U </span>
				<input
					type="checkbox"
					bind:checked={instance.states[state].underline}
					disabled={!instance.action.user_title_enabled}
					class="mr-4 mt-1 scale-125"
				/>
				<span class="mr-2"> Size </span>
				<!-- The type property is spread so Svelte does not convert the value to a number. -->
				<input
					{...{ type: "number" }}
					bind:value={instance.states[state].size}
					class="px-0.5 w-14 dark:text-neutral-300 dark:bg-neutral-600 rounded-md outline-none"
				/>
			</div>
		</div>
	</div>
</div>
