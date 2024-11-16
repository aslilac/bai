import ReactPlugin from "@vitejs/plugin-react";

export default {
	root: "src/",
	plugins: [ReactPlugin({ babel: { plugins: ["react-compiler"] } })],
	build: {
		emptyOutDir: true,
		outDir: "../build/",
		target: "es2022",
	},
} satisfies import("vite").UserConfig;
