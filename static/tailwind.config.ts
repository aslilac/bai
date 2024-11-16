export default {
	content: ["./index.html", "./src/**/*.tsx"],
	theme: {
		fontFamily: {
			sans: ["Outfit", "sans-serif"],
			serif: ["Merriweather", "serif"],
			mono: ['"Cascadia Code"', "monoscape"],
			hand: ["Reenie Beanie"],
		},
		extend: {
			gridTemplateColumns: {
				sidebar: "min-content auto",
			},
			transitionProperty: {
				bg: "background-color",
				spacing: "margin, padding",
			},
		},
	},
	plugins: [],
} satisfies import("tailwindcss").Config;
