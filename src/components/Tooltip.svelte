<script>
	let show = false;
</script>

<div
	class="inline-block cursor-help"
	on:mouseenter={() => show = true}
	on:mouseleave={() => show = false}
	role="cell" tabindex="-1"
>
	<img src="/question.png" class="w-6" alt="Help" />
	{#if show}
		<div class="absolute mt-2 p-2 w-96 text-xs bg-gray-100 border-2 rounded-lg">
			<slot />
		</div>
	{/if}
</div>
