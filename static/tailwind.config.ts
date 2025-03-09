export default {
	content: ["./index.html", "./src/**/*.tsx"],
	theme: {
		fontFamily: {
			sans: ["Outfit", "sans-serif"],
			serif: ["Merriweather", "serif"],
			mono: ['"Cascadia Code"', "monoscape"],
		},
	},
	plugins: [],
} satisfies import("tailwindcss").Config;
