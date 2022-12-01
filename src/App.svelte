<script lang="ts">
	import { listen, emit } from "@tauri-apps/api/event";
	import { event, invoke } from "@tauri-apps/api";
	import { onMount } from "svelte";
	import PickPort from "./lib/PickPort.svelte";
	import CircleSector from "./lib/CircleSector.svelte";
	import RadarAnimation from "./lib/RadarAnimation.svelte";
	import Cursor from "./lib/Cursor.svelte";
	import { slide } from "svelte/transition";
	import { appWindow } from '@tauri-apps/api/window';
	// import { emit } from "@tauri-apps/api/helpers/event";

	const missileAudio = [new Audio("sound/missil1.mp3"), new Audio("sound/missil2.mp3"), new Audio("sound/missil3.mp3")];
	const missAudio = [new Audio("sound/miss1.mp3"), new Audio("sound/miss2.mp3"), new Audio("sound/miss3.mp3")];
	const hitAudio = [new Audio("sound/hit1.mp3"), new Audio("sound/hit2.mp3"), new Audio("sound/hit3.mp3")];
	let isAudioPlaying = false;

	enum JoystickDirections {
		Up,
		Right,
		Down,
		Left,
	}

	enum GameState {
		PreSetup,
		Setup,
		WaitSetup,
		YourTurn,
		OtherTurn,
		Win,
		Defeat,
	}



	let gameStates = ["PreSetup", "Setup", "WaitSetup", "YourTurn", "OtherTurn", "Win", "Defeat"];

	interface Cell {
		index: number;
		ship: boolean;
		hit: boolean;
	}

	interface Cursor {
		x: number;
		y: number;
	}

	let gameState = GameState.PreSetup;
	let port_connected = false;
	let rows = 10;
	let cols = 10;
	let myBoard = [];
	let theirBoard: Cell[] = [];
	let cursorPosition = 0;
	let showDebug = false;
	let showMyBoard = false;
	let showDirectionButtons = false;
	let shipSizes = [2, 3, 3, 4, 5];
	let boardSize = 800;
	let boardGap = 5;
	let cursor = { x: 0, y: 0 };
	let debugText = "No errors";
	let devices = [];
	let selectedDevice;
	let cameraIndex = 0;

	createEmptyBoards();

	function getCellClasses(cell, cursorPosition) {
		return (cell.index == cursorPosition ? "selected-cell " : " ") + (cell.ship ? "ship-cell " : " ") + (cell.hit ? "hit-cell " : " ");
	}

	function setGameState(state) {
		console.log(state);
		switch (state) {
				case "PreSetup":
					gameState = GameState.PreSetup;
					break;
				case "Setup":
					gameState = GameState.Setup;
					break;
				case "WaitSetup":
					gameState = GameState.WaitSetup;
					break;
				case "YourTurn":
					gameState = GameState.YourTurn;
					break;
				case "OtherTurn":
					gameState = GameState.OtherTurn;
					break;
				case "Win":
					gameState = GameState.Win;
					break;
				case "Defeat":
					gameState = GameState.Defeat;
					break;
				default:
					break;
		}
	}

	onMount(async () => {
		const unlistenBoard = await listen<Boolean[]>("board-state", (event) => {
			for (let i = 0; i < event.payload.length; i++) {
				myBoard[i].ship = event.payload[i];
			}
		});

		boardSize = window.innerWidth * 0.45	;

		// const unlistenJoystick = await listen<Number>("joystick_direction", (event) => {
		// 	moveCursor(event.payload as JoystickDirections);
		// });

		await listen<number>("enemy-board-hit", (event) => {
			console.log("enemy-board-hit");
			let missile = randElem(missileAudio);
			missile.addEventListener("ended", () => {
				let hitSound = randElem(hitAudio);
				hitSound.addEventListener("ended", () => {
					isAudioPlaying = false;
				}, { once: true });
				hitSound.play();
				theirBoard[event.payload].hit = true;
				theirBoard[event.payload].ship = true;
				theirBoard = theirBoard;
			}, { once: true });
			isAudioPlaying = true;
			missile.play();
		});
		await listen<number>("enemy-board-miss", (event) => {
			console.log("enemy-board-miss");
			let missile = randElem(missileAudio);
			missile.addEventListener("ended", () => {
				let missSound = randElem(missAudio);
				missSound.addEventListener("ended", () => {
					isAudioPlaying = false;
				}, { once: true });
				missSound.play();
				theirBoard[event.payload].hit = true;
				theirBoard = theirBoard;
			}, { once: true });
			isAudioPlaying = true;
			missile.play();
		});
		await listen<number>("my-board-hit", (event) => {
			console.log("my-board-hit");
			myBoard[event.payload].hit = true;
			myBoard = myBoard;
		});
		await listen<string>("game-state", (event) => {
			setGameState(event.payload);
		});
		await listen<number>("update-cursor-pos", (event) => {
			cursorPosition = event.payload;
		});
		await listen<Cursor>("update-2d-cursor-pos", (event) => {
			console.log(event.payload);
			cursor = event.payload;
		});

		await listen<string>("error", (event) => {
			debugText = event.payload;
			console.log(event.payload);
		})

		await invoke("set_cursor_pos", { newPos: cursorPosition });
		await invoke("set_cols", { newCols: cols });
		await invoke("set_rows", { newRows: rows });
		
		getCaptureDevices();

		// setCaptureDevice(devices[1].deviceId);
		// console.log(devices);
		// const video: HTMLMediaElement = document.querySelector('#video');
		// video = videoStream;
		// $: if (stream && video.srcObject !== stream) {
		// video.srcObject = stream;
		// }
	});

	async function getCaptureDevices() {
		const stream = await navigator.mediaDevices.getUserMedia({ video: true });
		let mediaDevices = await navigator.mediaDevices.enumerateDevices();

		devices = mediaDevices;
		if (devices.length > 0) {
			selectedDevice = devices[0];
			setCaptureDevice(selectedDevice.deviceId);
		}
	}

	async function setCaptureDevice(deviceId) {
		const video: HTMLMediaElement = document.querySelector('#video');
		const stream = await navigator.mediaDevices.getUserMedia({
			video: {
				deviceId: {
					exact: deviceId
				}
			}
		});
		video.srcObject = stream;

		cameraIndex = devices.findIndex(x => x.deviceId == deviceId);
	}

	function stopCaptureDevice() {
		// const stream = await navigator.mediaDevices.getUserMedia({
		// 	video: {
		// 		deviceId: {
		// 			exact: deviceId
		// 		}
		// 	}
		// });
		// stream.stop();
		const video: HTMLMediaElement = document.querySelector('#video');
		video.srcObject = null;
	}


	

	async function startGame() {
		createEmptyBoards();
		stopCaptureDevice();
		await invoke("run_game", { shipSizes: shipSizes, camId: cameraIndex }).catch(error => {
			console.log(error);
			debugText = error;
		});
	}

	async function restartGame() {
		await emit("restart_game").then(() => gameState = GameState.PreSetup);
		// await emit("restart_game").then(() => startGame());
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
		emit("fire");
	}

	function randElem(arr) {
		return arr[Math.floor(Math.random() * arr.length)]
	}

	function getCellColor(cell) {
		return cell.ship && cell.hit ? "#dc143c" : cell.hit ? "blue" : "black";
	}

	function getHoverColor(cell) {
		return cell.ship && cell.hit ? "#9a0e2a" : cell.hit ? "blue" : "#05FB11";
	}

	function getGameStateText(gameState, port_connected) {
		switch (gameState) {
			case GameState.PreSetup:
				if (port_connected) {
					return "Ready to start the game";
				} else {
					return "Please connect the controller";
				}
			case GameState.Setup:
				return "Press fire to confirm ship positions";
			case GameState.WaitSetup:
				return "Wait for the other player to place their ships";
			case GameState.YourTurn:
				return "Your turn";
			case GameState.OtherTurn:
				return "";
			case GameState.Win:
				return "";
			case GameState.Defeat:
				return "";
		}
	}

	let debug = false;
	function toggleDebug() {
		debug = !debug;
	}
</script>

<main>
	<div class="fixed top-5 right-5 flex flex-col gap-2 text-left z-[100]">
		<button on:click={restartGame} class="">Restart</button>
		<button on:click={toggleDebug} class="">Debug</button>
		<button on:click={() => appWindow.close()} class="">Close</button>
	</div>
	{#if debug}
		<div class=" fixed top-5 left-5 flex flex-col text-left z-[100]">
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
			<label class="w-fit">
				Size
				<input type="range" name="" id="" min="400" max="1000" bind:value={boardSize} />
				{boardSize}
			</label>
			<span class=" text-red-700">{debugText}</span>
			<span>Camera index: {cameraIndex}</span>
			<!-- <label for="gamestate" class="flex flex-col text-left">
				Gamestate:
				<select bind:value={gameState} name="gamestate" class="h-9 p-2 rounded-md">
					{#each gameStates as gs}
						<option value={setGameState(gs)}>{gs}</option>
					{/each}
				</select>
			</label> -->
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
	{/if}
	
	

	<h1 style="max-width: {boardSize}; height: 57px;" class="mb-5" transition:slide>{getGameStateText(gameState, port_connected)}</h1>

	<!-- Overlay when waiting for turn -->
	{#if gameState == GameState.OtherTurn && !isAudioPlaying}
		<div style="background-color: rgba(0,0,0,0.5);" class="pop-up fixed w-full h-full grid items-center top-0 left-0 z-[80]">
			<h1>Wait for enemy turn</h1>
		</div>
	{/if}

	<!-- Overlay when game ends -->
	{#if (gameState == GameState.Win || gameState == GameState.Defeat) && !isAudioPlaying}
		<div style="background-color: rgba(0,0,0,0.5);" class="pop-up fixed w-full h-full grid items-center top-0 left-0 z-[80]">
			{#if gameState == GameState.Win}
				<h1 class=" font-bold">Congratulations you won!</h1>
			{:else if gameState == GameState.Defeat}
				<h1 class=" font-bold">Defeat :(</h1>
			{/if}
		</div>
	{/if}

	<div style="filter: blur({((gameState == GameState.OtherTurn || gameState == GameState.Win || gameState == GameState.Defeat) && !isAudioPlaying) ? 5 : 0}px);" class="grid place-content-center game-area">
		{#if gameState == GameState.PreSetup}
			{#if !port_connected}
				<PickPort bind:port_connected />
			{:else}
				<button on:click={startGame}>Start game</button>
			{/if}
			<div>
				<label for="devices" class="flex flex-col text-left mb-4 mt-4">
					Camera:
					<div class="grid grid-cols-[1fr,auto] gap-3">
						<select bind:value={selectedDevice} on:change={() => setCaptureDevice(selectedDevice.deviceId)} name="ports" class="h-9 p-2 rounded-md">
							{#each devices as device}
							<option value={device}>{device.label}</option>
							{/each}
						</select>
						<button on:click={() => getCaptureDevices()} class="pt-2 pb-2 h-9 leading-none">Refresh</button>
					</div>
				</label>
				<video autoplay id="video"></video>
			</div>
		{:else if gameState == GameState.Setup || gameState == GameState.WaitSetup}
			<div style="grid-template-columns: repeat({cols}, auto); grid-template-rows: repeat({rows}, auto);" class="board my grid gap-2 w-fit">
				{#each myBoard as cell, i}
					<div style="width: {window.innerHeight * 0.07}px; height: {window.innerHeight * 0.07}px;" class="text-sm bg-blue rounded-xl grid content-center shadow-md {getCellClasses(cell, cursorPosition)}">
						{#if showDebug}
							<p>{cell.index}</p>
							<p>Ship: {cell.ship}</p>
							<p>Hit: {cell.hit}</p>
						{/if}
					</div>
				{/each}
			</div>
		{:else}
			<div class="game w-fit">
				{#if showMyBoard}
				<div style="grid-template-columns: repeat({cols}, auto); grid-template-rows: repeat({rows}, auto);" class="board my grid gap-2">
					{#each myBoard as cell, i}
							<div class="w-20 h-20 text-sm bg-blue rounded-xl grid content-center shadow-md {getCellClasses(cell, cursorPosition)}">
								{#if showDebug}
								<p>{cell.index}</p>
								<p>Ship: {cell.ship}</p>
								<p>Hit: {cell.hit}</p>
								{/if}
							</div>
						{/each}
					</div>
				{/if}
				
				<div style="transform: translate({0}px, {0}px); width: {boardSize}px; height: {boardSize}px;" class=" relative radar">
					{#each theirBoard as cell, i}
						<CircleSector
							width={(0.5 * boardSize) / (rows+1) - boardGap}
							color={getCellColor(cell)}
							hoverColor={getHoverColor(cell)}
							sections={cols}
							radius={((0.5 * boardSize) / (rows+1)) * (2 + Math.floor(i / cols))}
							n={i % cols}
							letter={Math.floor(i / cols)}
							gap={boardGap}
							center={boardSize / 2}
							selected={cursorPosition == i}
						/>
					{/each}
					<div class="z-20"><RadarAnimation size={boardSize} /></div>
					<!-- <div style="top: {boardSize - (cursor.y + 1) * 0.5 * boardSize}px; left: {(cursor.x + 1) * 0.5 * boardSize}px;" class="cursor z-50" /> -->
					<Cursor bind:boardSize bind:x={cursor.x} bind:y={cursor.y}></Cursor>
				</div>
			</div>
		{/if}
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
		background-color: #dc143c;
	}
	.my .ship-cell {
		background-color: rgb(128, 130, 133);
	}
	.their .hit-cell {
		background-color: rgb(63, 65, 68);
	}
	/* .cursor {
		position: absolute;
		background-color: white;
		width: 10px;
		height: 10px;
		border-radius: 100%;
		transform: translate(-5px, -5px);
	} */
	.game-area {
		transition: filter 0.5s;
		transition-timing-function: ease-in;
	}

	.pop-up {
		transition: all 0.5s;
		transition-timing-function: ease-in;
	}

	.radar {
		background-color: #05fb11;
		border-radius: 50%;
		border: 5px solid #05fb11;
		box-sizing: content-box;
	}
</style>
