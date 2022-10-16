<script lang="ts">
	let rows = 3;
	let cols = 3;
	let myBoard = [];
	let theirBoard = [];
	let cursorPosition = 0;
	let showDebug = false;
	let showMyBoard = false;

	for (let i = 0; i < rows * cols; i++) {
		myBoard.push({ index: i, ship: Boolean(i % 3), hit: Boolean(i % 2) });
	}
	for (let i = 0; i < rows * cols; i++) {
		theirBoard.push({ index: i, ship: Boolean(i % 3), hit: Boolean(i % 2) });
	}
	// console.log(myBoard);

	function getCellClasses(cell) {
		return (cell.index == cursorPosition ? "selected-cell " : " ") + (cell.ship ? "ship-cell " : " ") + (cell.hit ? "hit-cell " : " ");
	}
</script>

<main>
	<div class=" fixed top-10 left-10 flex flex-col">
		<label>
			Show debug info
			<input type="checkbox" name="debugInfo" bind:checked={showDebug} class=" w-4 h-4 ml-2" />
		</label>
		<label>
			Show own board
			<input type="checkbox" name="debugInfo" bind:checked={showMyBoard} class=" w-4 h-4 ml-2" />
		</label>
	</div>
	<div class="game">
		{#if showMyBoard}
			<div style="grid-template-columns: repeat({cols}, auto); grid-template-rows: repeat({rows}, auto);" class="board my grid gap-2">
				{#each myBoard as cell, i}
					<div class="w-20 h-20 text-sm bg-slate-500 rounded-xl grid content-center shadow-md {getCellClasses(cell)}">
						{#if showDebug}
							<p>{cell.index}</p>
							<p>Ship: {cell.ship}</p>
							<p>Ship: {cell.hit}</p>
						{/if}
					</div>
				{/each}
			</div>
		{/if}
		<div style="grid-template-columns: repeat({cols}, auto); grid-template-rows: repeat({rows}, auto);" class="board their grid gap-2">
			{#each theirBoard as cell, i}
				<div class="w-20 h-20 text-sm bg-slate-500 rounded-xl grid content-center shadow-md {getCellClasses(cell)}">
					{#if showDebug}
						<p>{cell.index}</p>
						<p>Ship: {cell.ship}</p>
						<p>Ship: {cell.hit}</p>
					{/if}
				</div>
			{/each}
		</div>
	</div>
</main>

<style>
	.game {
		display: grid;
		grid-auto-flow: column;
		grid-gap: 150px;
	}
	.their .selected-cell {
		border: solid 3px rgb(255, 255, 255);
	}
	main .ship-cell.hit-cell {
		background-color: crimson;
	}
	.my .ship-cell {
		background-color: rgb(128, 130, 133);
	}
	.their .hit-cell {
		background-color: rgb(63, 65, 68);
	}
</style>
