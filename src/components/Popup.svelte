<script lang="ts">
	export let show = false;
</script>

{#if show}
	<div class="absolute top-0 left-0 m-2 p-4 w-[calc(100%-1rem)] h-[calc(100%-1rem)] bg-neutral-100 dark:bg-neutral-800 border-2 dark:border-neutral-700 rounded-lg overflow-scroll z-30">
		<slot />
	</div>
{/if}
