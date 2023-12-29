<script lang="ts">
	export let show = false;
</script>

<div
	class="absolute top-0 left-0 m-4 p-4 w-[calc(100%-2rem)] h-[calc(100%-2rem)] bg-gray-100 border-2 rounded-lg"
	class:hidden={!show}
>
	<slot />
</div>
