import ReactPlugin from "@vitejs/plugin-react";
import TailwindPlugin from "@tailwindcss/vite";
import tsconfigPaths from "vite-tsconfig-paths";

export default {
	root: "src/",
	plugins: [
		ReactPlugin({ babel: { plugins: ["react-compiler"] } }),
		TailwindPlugin(),
		tsconfigPaths(),
	],
	build: {
		emptyOutDir: true,
		outDir: "../build/",
		target: "es2022",
	},
} satisfies import("vite").UserConfig;
