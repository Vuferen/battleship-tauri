<script>
	import { onMount } from "svelte";

	export let width;
	export let color;
	export let hoverColor;
	export let sections;
	export let n;
	export let radius;
	export let gap;
	export let center;
	export let selected;
	export let letter;
	// let newRadius = radius - width * 0.5 + (gap * (radius / width - 1) + 0.5 * gap);
	$: newRadius = radius - width*0.5 - gap*2;
	$: size = newRadius * 2 + width * 2;
	$: circumference = newRadius * Math.PI * 2;

	let angle = ((Math.PI * 2) / sections) * 0.5;

	let x = (Math.sin(angle) * (gap * sections)) / 5;
	let y = (Math.cos(angle) * (gap * sections)) / 5;
	// console.log("x: " + textX + ", y: " + textY);

	function getColor(select, color, hoverColor) {
		return selected ? hoverColor : color;
	}

	function getLetter(n) {
		switch (n) {
			case 0:
				return "A"
			case 1:
				return "B"
			case 2: 
				return "C"
			case 3:
				return "D"
			case 4:
				return "E"
			case 5:
				return "F"
			case 6:
				return "G"
			case 7:
				return "H"
			case 8:
				return "I"
			case 9:
				return "J"			
			default:
				break;
		}
	}

</script>
<!-- -newRadius + center / 2 + gap*sections*2-gap -->
<div
	style="width: {size}px; height: {size}px; top: {-size*0.5+center}px; left: {-size*0.5+center}px; transform: rotate({(360 /
		sections) *
		n +
		angle}deg) translate({x}px, {-y}px);"
	class=" absolute"
>

	<svg viewBox="0 0 {size} {size}">
		<path id="curve"
		d="M{newRadius + width} {width}
		a {newRadius} {newRadius} 0 0 1 0 {newRadius * 2}
		a {newRadius} {newRadius} 0 0 1 0 -{newRadius * 2}"
		fill="none"
		stroke={getColor(selected, color, hoverColor)}
		stroke-width={width}
		stroke-dasharray="{circumference / sections}, {circumference}"
		/>
		<!-- <text width="{size}">
			<textPath xlink:href="#curve" startOffset="50%" text-anchor="middle">
				{getLetter(letter)}{n+1}
			</textPath>
		</text> -->
	</svg>
		<!-- <p class=" z-20 absolute">{getLetter(letter)}{n+1}</p> -->
</div>
<!-- {#if selected}
	<div
		style="width: {size}px; height: {size}px; top: {-size*0.5+center}px; left: {-size*0.5+center}px; transform: rotate({(360 /
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
{/if} -->
<div
	style="width: {size}px; height: {size}px; top: {-size*0.5+center}px; left: {-size*0.5+center}px; transform: rotate({(360 /
		sections) *
		n +
		angle + (360/sections)*0.5}deg) translate({0}px, {width*0.5}px);"
	class=" absolute z-20"
>
<span>{letter+1}</span>
</div>

{#if letter == 0}
	<div
	style="width: {size}px; height: {size}px; top: {-size*0.5+center}px; left: {-size*0.5+center}px; transform: rotate({(360 /
		sections) *
		n +
		angle + (360/sections)*0.5}deg) translate({0}px, {newRadius-center}px);"
	class=" absolute z-20"
	>
		<span>{getLetter(n)}</span>
	</div>
{/if}

{#if letter == 0 && n == 0}
<div
	style="width: {gap*2}px; height: {center+gap}px; top: {-gap}px; left: {center-gap}px; background-color: #05FB11; border-radius: {gap}px;"
	class=" absolute z-10"
	>
	</div>
{/if}

<style>
</style>
