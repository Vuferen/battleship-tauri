<script>
	export let size;
</script>

<div style="height: {size+2}px; border-radius: {size}px;" class="radar-animation">
	<div style="height: {size*0.5+1}px; transform: translateX({size*0.5-2.5}px);" class="radar-line">

	</div>
	<div style="height: {size*0.5+1}px; transform: translateX({size*0.5}px);" class="radar-gradient">

	</div>
</div>

<style>
	.radar-gradient {
		position: absolute;
		left: -500px;
		width: 500px;
		background: linear-gradient(45deg, rgba(0,0,0,0) 50%, #05FB11 100%);
	}
	.radar-line {
		position: absolute;
		width: 5px;
		background-color: #05FB11;
	}
	.radar-animation{
		position: relative;
		overflow: hidden;
		
		/* box-sizing: content-box; */
		/* background-color: linear-gradient(#05FB11, black); */
		
		animation-name: rotate;
		animation-duration: 3s;
		animation-iteration-count: infinite;
		animation-timing-function: linear;
	}
	@keyframes rotate {
		from {transform: rotate(0deg);}
		to {transform: rotate(360deg);}
	}
</style>