<script lang="ts">
	import { onMount } from "svelte";
	import { invoke } from "@tauri-apps/api";

	let ports = [];
	let selected;
	let baudrate = 9600;
	let errorMessage = "";

	onMount(async () => {
		await invoke("get_ports")
			.then(async (portNames: String[]) => {
				ports = portNames;
				// Auto select first port
				if (ports.length > 0) {
					selected = ports[ports.length-1];
					getPort(selected);
				}
			})
			.catch((err) => console.log(err));
	});

	async function getPort(port) {
		errorMessage = "";
		await invoke("pick_port", { portName: port, baudrate: baudrate }).catch((err) => (errorMessage = err));
	}
</script>

<div>
	<select bind:value={selected} on:change={() => getPort(selected)} name="ports">
		{#each ports as port}
			<option value={port}>{port}</option>
		{/each}
	</select>
	<input type="number" name="baudrate" id="" bind:value={baudrate} />
	{#if errorMessage != ""}
		<span class=" text-red-700">{errorMessage}</span>
	{/if}
</div>
