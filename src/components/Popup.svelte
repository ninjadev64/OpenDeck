<script lang="ts">
	export let show = false;
</script>

{#if show}
	<div class="absolute top-0 left-0 m-2 p-4 w-[calc(100%-1rem)] h-[calc(100%-1rem)] bg-gray-100 border-2 rounded-lg overflow-scroll">
		<slot />
	</div>
{/if}
