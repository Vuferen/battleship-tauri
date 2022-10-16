<script lang="ts">
	import { listen } from "@tauri-apps/api/event";
	import { invoke } from "@tauri-apps/api";
	import { onMount } from "svelte";
	import PickPort from "./lib/PickPort.svelte";

	enum JoystickDirections {
		Up,
		Right,
		Down,
		Left,
	}

	let rows = 3;
	let cols = 3;
	let myBoard = [];
	let theirBoard = [];
	let cursorPosition = 0;
	let showDebug = false;
	let showMyBoard = false;
	let showDirectionButtons = false;

	for (let i = 0; i < rows * cols; i++) {
		myBoard.push({ index: i, ship: Boolean(i % 3), hit: Boolean(i % 2) });
	}
	for (let i = 0; i < rows * cols; i++) {
		theirBoard.push({ index: i, ship: Boolean(i % 3), hit: Boolean(i % 2) });
	}
	// console.log(myBoard);

	function getCellClasses(cell, cursorPosition) {
		return (cell.index == cursorPosition ? "selected-cell " : " ") + (cell.ship ? "ship-cell " : " ") + (cell.hit ? "hit-cell " : " ");
	}
	onMount(async () => {
		// const unlistenPorts = await listen<String[]>("available_ports", (event) => {
		// 	ports = event.payload;
		// 	console.log(ports);
		// });
		// await invoke("get_ports")
		// 	.then((portNames: String[]) => (ports = portNames))
		// 	.catch((err) => console.log(err));

		// await invoke("pick_port", { port: "COM3", baudrate: 9600 });

		const unlistenBoard = await listen<Boolean[]>("board-state", (event) => {
			for (let i = 0; i < event.payload.length; i++) {
				myBoard[i].ship = event.payload[i];
			}
		});

		const unlistenJoystick = await listen<Number>("joystick_direction", (event) => {
			moveCursor(event.payload as JoystickDirections);
		});
	});

	function moveCursor(direction: JoystickDirections) {
		switch (direction) {
			case JoystickDirections.Up:
				cursorPosition += cursorPosition - cols < 0 ? cols * (rows - 1) : -cols;
				break;
			case JoystickDirections.Right:
				cursorPosition += (cursorPosition + 1) % cols ? 1 : -cols + 1;
				break;
			case JoystickDirections.Down:
				cursorPosition += cursorPosition + cols > cols * rows - 1 ? -cols * (rows - 1) : cols;
				break;
			case JoystickDirections.Left:
				cursorPosition += cursorPosition % cols ? -1 : cols - 1;
				break;

			default:
				break;
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

		<div class="mt-4">
			{#if showDirectionButtons}
				<div class="mb-5">
					<button on:click={() => moveCursor(JoystickDirections.Up)}>Up</button>
					<button on:click={() => moveCursor(JoystickDirections.Right)}>Right</button>
					<button on:click={() => moveCursor(JoystickDirections.Down)}>Down</button>
					<button on:click={() => moveCursor(JoystickDirections.Left)}>Left</button>
				</div>
			{/if}
		</div>
	</div>

	<div class="game w-fit">
		{#if showMyBoard}
			<div style="grid-template-columns: repeat({cols}, auto); grid-template-rows: repeat({rows}, auto);" class="board my grid gap-2">
				{#each myBoard as cell, i}
					<div class="w-20 h-20 text-sm bg-slate-500 rounded-xl grid content-center shadow-md {getCellClasses(cell, cursorPosition)}">
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
				<div class="w-20 h-20 text-sm bg-slate-500 rounded-xl grid content-center shadow-md {getCellClasses(cell, cursorPosition)}">
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
