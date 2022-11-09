<script lang="ts">
	import { onMount } from "svelte";
	import { invoke } from "@tauri-apps/api";

	let ports = [];
	let baudrates = [
		50, 75, 110, 134, 150, 200, 600, 1200, 1800, 2400, 4800, 9600, 19200, 28800, 38400, 57600, 76800, 115200, 230400, 230400, 460800, 576000,
		921600,
	];
	let selected;
	let baudrate = 9600;
	let errorMessage = "";
	let refreshButtonLabel = "Refresh";
	export let port_connected = false;

	onMount(async () => {
		getAvailablePorts();
	});

	async function getPort(port) {
		errorMessage = "";
		await invoke("pick_port", { portName: port, baudrate: baudrate })
			.then(() => {
				port_connected = true;
			})
			.catch((err) => {
				errorMessage = err;
				port_connected = false;
			});
	}

	async function getAvailablePorts() {
		await invoke("get_ports")
			.then(async (portNames: String[]) => {
				ports = portNames;
				// Auto select first port
				if (ports.length > 0) {
					// selected = ports[ports.length-1];
					selected = ports[0];
					getPort(selected);
				}
			})
			.catch((err) => console.log(err));
	}

	async function refreshPorts() {
		refreshButtonLabel = "Refreshing";
		await getAvailablePorts();
		refreshButtonLabel = "Refreshed";
		setTimeout(() => {
			refreshButtonLabel = "Refresh";
		}, 1000);
	}
</script>

<div class="flex flex-col items-center">
	<p class=" text-red-700 h-7">{errorMessage}</p>
	<div class="flex flex-col">
		<div class="mt-4 flex flex-row gap-3 items-end">
			<label for="ports" class="flex flex-col text-left">
				Port:
				<select bind:value={selected} on:change={() => getPort(selected)} name="ports" class="h-9 p-2 rounded-md">
					{#each ports as port}
						<option value={port}>{port}</option>
					{/each}
				</select>
			</label>

			<label for="baudrate" class="flex flex-col text-left">
				Baudrate:
				<select bind:value={baudrate} name="baudrate" class="h-9 p-2 rounded-md">
					{#each baudrates as baudrate}
						<option value={baudrate}>{baudrate}</option>
					{/each}
				</select>
				<!-- <input type="number" name="baudrate" id="" bind:value={baudrate} class="h-9 p-2 rounded-md" /> -->
			</label>

			<button on:click={() => refreshPorts()} class="pt-2 pb-2 h-9 leading-none">{refreshButtonLabel}</button>
		</div>
		<div class="mt-5 flex gap-2 justify-end">
			<button on:click={() => getPort(selected)} class=" w-full bg-main hover:bg-main-hover focus:bg-main-hover text-black font-bold"
				>Connect</button
			>
		</div>
	</div>
</div>
