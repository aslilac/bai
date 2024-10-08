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
			"test/$$name$$_test.gleam",
			"test/$$name$$/example_test.gleam",
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
			".github/workflows/main.yml@node",
			".gitignore",
			".prettierignore",
			".prettierrc.json",
			"CODE_OF_CONDUCT.md",
			"LICENSE",
			"src/index.ts",
			"package.json",
			"README.md",
			"tsconfig.build.json",
			"tsconfig.json",
		]),
		"/react" | "/tsx" => Ok(&[
			".github/workflows/main.yml@node",
			".gitignore",
			".prettierignore",
			".prettierrc.json",
			"CODE_OF_CONDUCT.md",
			"LICENSE",
			"package.json@react",
			"README.md",
			"src/$$name$$.tsx",
			"tsconfig.json@react",
		]),
		any_other_group => Err(anyhow!("unrecognized group: {}", any_other_group)),
	}
}
