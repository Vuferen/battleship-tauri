<script lang="ts">
	import { onMount } from "svelte";
	import { invoke } from "@tauri-apps/api";

	let ports = [];
	let selected;
	let baudrate = 9600;

	onMount(async () => {
		// const unlistenPorts = await listen<String[]>("available_ports", (event) => {
		// 	ports = event.payload;
		// 	console.log(ports);
		// });
		await invoke("get_ports")
			.then(async (portNames: String[]) => {
				ports = portNames;
				if (ports.length > 0) {
					getPort(ports[0]);
				}
			})
			.catch((err) => console.log(err));
	});

	async function getPort(port) {
		await invoke("pick_port", { port: port, baudrate: baudrate }).catch((err) => console.log(err));
	}
</script>

<div>
	<select bind:value={selected} on:change={() => getPort(selected)} name="ports">
		{#each ports as port}
			<option value={port}>{port}</option>
		{/each}
	</select>
	<input type="number" name="baudrate" id="" bind:value={baudrate} />
</div>
