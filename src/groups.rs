use anyhow::anyhow;

pub fn expand_group<S: AsRef<str>>(group: S) -> anyhow::Result<&'static [&'static str]> {
	match group.as_ref() {
		"/gleam" => Ok(&[
			".github/workflows/main.yml@gleam",
			".gitignore",
			"CODE_OF_CONDUCT.md",
			"LICENSE",
			"gleam.toml",
			"README.md",
			"src/$$name$$.gleam",
		]),
		"/go" => Ok(&[
			".github/workflows/main.yml@go",
			".gitignore",
			"CODE_OF_CONDUCT.md",
			"LICENSE",
			"go.mod",
			"main.go",
			"README.md",
			"staticcheck.conf",
		]),
		"/rs" | "/rust" => Ok(&[
			".cargo/config.toml",
			".github/workflows/main.yml@rust",
			".gitignore",
			".rustfmt.toml",
			"Cargo.toml",
			"CODE_OF_CONDUCT.md",
			"LICENSE",
			"README.md",
			"rust-toolchain.toml",
			"src/main.rs",
			"tests/main.rs",
			"tests/setup.rs",
		]),
		"/ts" | "/typescript" => Ok(&[
			".github/workflows/main.yml",
			".eslintrc.json",
			".gitignore",
			".prettierignore",
			".prettierrc.json",
			"CODE_OF_CONDUCT.md",
			"LICENSE",
			"jest.config.js",
			"package.json",
			"README.md",
			"tsconfig.build.json",
			"tsconfig.json",
		]),
		group => Err(anyhow!("unrecognized group: {}", group)),
	}
}
