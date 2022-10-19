<script>
	import { onMount } from "svelte";

	export let width;
	export let color;
	export let sections;
	export let n;
	export let radius;
	export let gap;
	export let center;
	export let selected;
	let newRadius = radius - width * 0.5 + (gap * (radius / width - 1) + 0.5 * gap);
	let size = newRadius * 2 + width * 2;
	let circumference = newRadius * Math.PI * 2;

	let angle = ((Math.PI * 2) / sections) * 0.5;

	let x = (Math.sin(angle) * (gap * sections)) / 5;
	let y = (Math.cos(angle) * (gap * sections)) / 5;

	function getColor(select, color) {
		return selected ? "#fff" : color;
	}
</script>

<div
	style="width: {size}px; height: {size}px; top: {-newRadius + center / 2}px; left: {-newRadius + center / 2}px; transform: rotate({(360 /
		sections) *
		n +
		angle}deg) translate({x}px, {-y}px);"
	class=" absolute"
>
	<svg viewBox="0 0 {size} {size}">
		<path
			d="M{newRadius + width} {width}
			a {newRadius} {newRadius} 0 0 1 0 {newRadius * 2}
			a {newRadius} {newRadius} 0 0 1 0 -{newRadius * 2}"
			fill="none"
			stroke={getColor(selected, color)}
			stroke-width={width}
			stroke-dasharray="{circumference / sections}, {circumference}"
		/>
	</svg>
</div>
{#if selected}
	<div
		style="width: {size}px; height: {size}px; top: {-newRadius + center / 2}px; left: {-newRadius + center / 2}px; transform: rotate({(360 /
			sections) *
			n +
			angle}deg) translate({x}px, {-y}px);"
		class=" absolute z-10"
	>
		<svg viewBox="0 0 {size} {size}">
			<path
				d="M{newRadius + width} {width}
			a {newRadius} {newRadius} 0 0 1 0 {newRadius * 2}
			a {newRadius} {newRadius} 0 0 1 0 -{newRadius * 2}"
				fill="none"
				stroke={color}
				stroke-width={width - 10}
				stroke-dasharray="{circumference / sections}, {circumference}"
			/>
		</svg>
	</div>
{/if}

<style>
</style>
