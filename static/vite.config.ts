import ReactPlugin from "@vitejs/plugin-react";
import TailwindPlugin from "@tailwindcss/vite";

export default {
	root: "src/",
	plugins: [
		ReactPlugin({ babel: { plugins: ["react-compiler"] } }),
		TailwindPlugin(),
	],
	build: {
		emptyOutDir: true,
		outDir: "../build/",
		target: "es2022",
	},
} satisfies import("vite").UserConfig;
