use anyhow::anyhow;
use once_cell::sync::Lazy;
use std::env;
use std::fs;
use std::path::Path;

mod options;

use options::Options;

const BASE: Lazy<reqwest::Url> = Lazy::new(|| {
	reqwest::Url::parse(
		"https://raw.githubusercontent.com/aslilac/mckayla/main/packages/create-ok/static/",
	)
	.unwrap()
});

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	let options = env::args().skip(1).collect::<Options>();

	let name = env::current_dir()?
		.file_name()
		.ok_or_else(|| anyhow!("expected current directory to have a name"))?
		.to_string_lossy()
		.to_string();

	for file in options.files {
		let (file_path, tag) = file
			.rsplit_once("@")
			.map(|(file_path, tag)| (file_path, Some(tag)))
			.unwrap_or((&file, None));

		let mut base = BASE.clone();
		if let Some(tag) = tag {
			base = base.join(&format!("@{}/", tag))?;
		}

		let file_content = reqwest::get(base.join(&file_path)?)
			.await?
			.error_for_status()?
			.text()
			.await?;

		let file_content = file_content.replace("{{name}}", &name);

		if let Some(parent) = Path::new(&file).parent() {
			fs::create_dir_all(parent)?;
		}
		fs::write(file_path, file_content)?;
	}

	// println!("{:?}", options);

	Ok(())
}
