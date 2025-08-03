use anyhow::anyhow;

const COMMON_FILES: &[&str] = &[".editorconfig", ".gitignore", "README.md"];

fn with_common_files(other_files: &[&'static str]) -> Vec<&'static str> {
	[COMMON_FILES, other_files].concat()
}

pub fn expand_group<S: AsRef<str>>(group: S) -> anyhow::Result<Vec<&'static str>> {
	match group.as_ref() {
		// Project templates
		"/gleam" => Ok(with_common_files(&[
			"Dockerfile@gleam",
			"gleam.toml",
			"src/$$name$$.gleam",
			"test/$$name$$_test.gleam",
			"test/$$name$$/example_test.gleam",
		])),
		"/go" => Ok(with_common_files(&[
			"go.mod",
			"main.go",
			"staticcheck.conf",
		])),
		"/rs" | "/rust" => Ok(with_common_files(&[
			".cargo/config.toml",
			".rustfmt.toml",
			"Cargo.toml",
			"rust-toolchain.toml",
			"src/main.rs",
			"tests/main.rs",
			"tests/setup.rs",
		])),
		"/ts" | "/typescript" => Ok(with_common_files(&[
			".prettierignore",
			".prettierrc.json",
			"package.json",
			"src/index.ts",
			"tsconfig.json",
			"tsconfig.build.json",
		])),
		"/react" | "/tsx" => Ok(with_common_files(&[
			".prettierignore",
			".prettierrc.json@react",
			"package.json@react",
			"src/index.css",
			"src/index.html",
			"src/index.tsx",
			"vite.config.ts",
			"tsconfig.json@react",
		])),

		// Addons for open-source repos
		"/oss" => Ok(vec![".github/FUNDING.yml", "CODE_OF_CONDUCT.md", "LICENSE"]),

		any_other_group => Err(anyhow!("unrecognized group: {}", any_other_group)),
	}
}

#[cfg(test)]
#[test]
fn group_files_exist() {
	use crate::parse_file_name;
	use std::fs;

	let groups = ["/foss", "/gleam", "/go", "/rs", "/ts", "/tsx"];
	for group in groups {
		let group = expand_group(group).unwrap();
		let files = group.iter().map(|it| {
			(
				it,
				parse_file_name(it)
					.unwrap()
					.1
					.path()
					.strip_prefix("/aslilac/bai/main/")
					.unwrap()
					.to_owned(),
			)
		});
		for (name, path) in files {
			assert!(
				fs::exists(&path).unwrap(),
				"missing file {} at {}",
				name,
				path
			);
		}
	}
}
