const config = {
	content: ["./src/**/*.{html,js,svelte,ts}"],

	theme: {
		extend: {
			colors: {
				main: "var(--main-color)",
				"main-hover": "var(--main-color-hover)",
				blue: "blue",
			},
		},
	},

	plugins: [],
};

module.exports = config;
