<script lang="ts">
	import { listen, emit } from "@tauri-apps/api/event";
	import { event, invoke } from "@tauri-apps/api";
	import { onMount } from "svelte";
	import PickPort from "./lib/PickPort.svelte";
	// import { emit } from "@tauri-apps/api/helpers/event";

	enum JoystickDirections {
		Up,
		Right,
		Down,
		Left,
	}

	enum GameState {
		Setup,
		Fire,
		End,
	}

	interface Cell {
		index: number;
		ship: boolean;
		hit: boolean;
	}

	let gameState = GameState.Setup;
	let rows = 3;
	let cols = 3;
	let myBoard = [];
	let theirBoard: Cell[] = [];
	let cursorPosition = 0;
	let showDebug = false;
	let showMyBoard = true;
	let showDirectionButtons = false;
	let shipSizes = [2, 2];
	let endMessage = "";

	createEmptyBoards();

	function getCellClasses(cell, cursorPosition) {
		return (cell.index == cursorPosition ? "selected-cell " : " ") + (cell.ship ? "ship-cell " : " ") + (cell.hit ? "hit-cell " : " ");
	}

	onMount(async () => {
		const unlistenBoard = await listen<Boolean[]>("board-state", (event) => {
			for (let i = 0; i < event.payload.length; i++) {
				myBoard[i].ship = event.payload[i];
			}
		});

		// const unlistenJoystick = await listen<Number>("joystick_direction", (event) => {
		// 	moveCursor(event.payload as JoystickDirections);
		// });

		await listen<number>("enemy-board-hit", (event) => {
			console.log("enemy-board-hit");
			theirBoard[event.payload].hit = true;
			theirBoard[event.payload].ship = true;
			theirBoard = theirBoard;
		});
		await listen<number>("enemy-board-miss", (event) => {
			console.log("enemy-board-miss");
			theirBoard[event.payload].hit = true;
			theirBoard = theirBoard;
		});
		await listen<number>("my-board-hit", (event) => {
			console.log("my-board-hit");
			myBoard[event.payload].hit = true;
			myBoard = myBoard;
		});
		await listen<boolean>("game-end", (event) => {
			console.log("game-end");
			if (event.payload) {
				endMessage = "Victory!";
			} else {
				endMessage = "Defeat :(";
			}
		});
		await listen<number>("update-cursor-pos", (event) => {
			cursorPosition = event.payload;
		});

		await invoke("set_cursor_pos", { newPos: cursorPosition });
		await invoke("set_cols", { newCols: cols });
		await invoke("set_rows", { newRows: rows });
	});

	async function runGame() {
		createEmptyBoards();
		gameState = GameState.Fire;
		await invoke("run_game", { shipSizes: shipSizes }).then(() => (gameState = GameState.End));
	}

	async function moveCursor(direction: JoystickDirections) {
		await invoke("move_cursor", { direction: direction });
	}

	function createEmptyBoards() {
		myBoard = [];
		theirBoard = [];
		for (let i = 0; i < rows * cols; i++) {
			myBoard.push({ index: i, ship: false, hit: false });
		}
		for (let i = 0; i < rows * cols; i++) {
			theirBoard.push({ index: i, ship: false, hit: false });
		}
	}

	function fire() {
		if (gameState != GameState.Fire) {
			endMessage = "";
			runGame();
		} else {
			emit("fire");
		}
	}
</script>

<main>
	<div class=" fixed top-10 left-10 flex flex-col text-left">
		<PickPort />

		<label class="mt-2 w-fit">
			Show debug info
			<input type="checkbox" name="debugInfo" bind:checked={showDebug} class=" w-4 h-4 ml-2" />
		</label>
		<label class="w-fit">
			Show own board
			<input type="checkbox" name="debugInfo" bind:checked={showMyBoard} class=" w-4 h-4 ml-2" />
		</label>
		<label class="w-fit">
			Show direction buttons
			<input type="checkbox" name="debugInfo" bind:checked={showDirectionButtons} class=" w-4 h-4 ml-2" />
		</label>
		<!-- <label class="w-fit mt-2 mb-2">
			<button on:click={confirmShips}>Confirm ship positions</button>
			
		</label> -->
		<div class="mt-4">
			<div class="mb-5">
				<button on:click={fire}>Fire</button>
				{#if showDirectionButtons}
					<button on:click={() => moveCursor(JoystickDirections.Up)}>Up</button>
					<button on:click={() => moveCursor(JoystickDirections.Right)}>Right</button>
					<button on:click={() => moveCursor(JoystickDirections.Down)}>Down</button>
					<button on:click={() => moveCursor(JoystickDirections.Left)}>Left</button>
				{/if}
			</div>
		</div>
	</div>

	<div class="game w-fit">
		<span>{endMessage}</span>
		{#if showMyBoard}
			<div style="grid-template-columns: repeat({cols}, auto); grid-template-rows: repeat({rows}, auto);" class="board my grid gap-2">
				{#each myBoard as cell, i}
					<div class="w-20 h-20 text-sm bg-slate-500 rounded-xl grid content-center shadow-md {getCellClasses(cell, cursorPosition)}">
						{#if showDebug}
							<p>{cell.index}</p>
							<p>Ship: {cell.ship}</p>
							<p>Hit: {cell.hit}</p>
						{/if}
					</div>
				{/each}
			</div>
		{/if}
		<div style="grid-template-columns: repeat({cols}, auto); grid-template-rows: repeat({rows}, auto);" class="board their grid gap-2">
			{#each theirBoard as cell, i}
				<div class="w-20 h-20 text-sm bg-slate-500 rounded-xl grid content-center shadow-md {getCellClasses(cell, cursorPosition)}">
					{#if showDebug}
						<p>{cell.index}</p>
						<p>Ship: {cell.ship}</p>
						<p>Hit: {cell.hit}</p>
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
