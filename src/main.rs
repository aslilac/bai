use anyhow::anyhow;
use chrono::Datelike;
use colored::Colorize;
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::HashMap;
use std::env;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::sync::Arc;
use tokio::task;

mod groups;
mod options;
mod regext;
use options::Options;

static BASE: Lazy<reqwest::Url> = Lazy::new(|| {
	reqwest::Url::parse("https://raw.githubusercontent.com/aslilac/okie/main/static/")
		.expect("invalid base URL")
});

const IDENT: Lazy<&str> = Lazy::new(|| include_str!("./ident.pcre").trim_end());
static TEMPLATE_VARIABLE: Lazy<Regex> =
	Lazy::new(|| Regex::new(&format!("\\{{\\{{ *{} *\\}}\\}}", *IDENT)).unwrap());
static PATH_TEMPLATE_VARIABLE: Lazy<Regex> =
	Lazy::new(|| Regex::new(&format!("\\$\\${}\\$\\$", *IDENT)).unwrap());

type Context = HashMap<String, String>;

fn parse_file_name(file: &str) -> anyhow::Result<(&str, reqwest::Url)> {
	let (file_path, tag) = file
		.rsplit_once("@")
		.map(|(file_path, tag)| (file_path, Some(tag)))
		.unwrap_or((&file, None));

	let mut base = BASE.clone();
	if let Some(tag) = tag {
		base = base.join(&format!("@{}/", tag))?;
	}

	Ok((file_path, base.join(&file_path)?))
}

async fn fetch_file<C>(file: &str, ctx: C) -> anyhow::Result<()>
where
	C: AsRef<Context>,
{
	let ctx = ctx.as_ref();
	let (file_path, url) = parse_file_name(file)?;

	// Fetch file
	let file_content = reqwest::get(url).await?.error_for_status()?.text().await?;

	// Create parent directories as necessary
	if let Some(parent) = Path::new(&file).parent() {
		if !parent.exists() {
			fs::create_dir_all(parent)?;
		}
	}

	let each = |captures: &regex::Captures| ctx.get(&captures[1]);
	// Fill in template variables
	let file_content = regext::for_each(&TEMPLATE_VARIABLE, file_content, each);
	let file_path = regext::for_each(&PATH_TEMPLATE_VARIABLE, file_path.to_string(), each);
	fs::write(file_path, file_content)?;

	Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	let Options { files, mut context } =
		Options::try_from(&*env::args().skip(1).collect::<Vec<_>>())?;

	if !context.contains_key(&"name".to_string()) {
		context.insert(
			"name".to_string(),
			env::current_dir()?
				.file_name()
				.ok_or_else(|| anyhow!("expected current directory to have a name"))?
				.to_string_lossy()
				.to_string(),
		);
	};

	if !context.contains_key(&"git.config.user.name".to_string()) {
		let output = Command::new("git")
			.args(["config", "user.name"])
			.output()
			.unwrap();

		if output.status.success() {
			// Ouch. Two allocations in one line.
			let stdout = String::from_utf8_lossy(&*output.stdout).trim().to_string();
			context.insert("git.config.user.name".to_string(), stdout);
		}
	};

	if !context.contains_key(&"git.config.user.email".to_string()) {
		let output = Command::new("git")
			.args(["config", "user.email"])
			.output()
			.unwrap();

		if output.status.success() {
			// Ouch. Two allocations in one line.
			let stdout = String::from_utf8_lossy(&*output.stdout).trim().to_string();
			context.insert("git.config.user.email".to_string(), stdout);
		}
	};

	if !context.contains_key(&"date.year".to_string()) {
		context.insert(
			"date.year".to_string(),
			chrono::Local::now().year().to_string(),
		);
	}

	let context = Arc::new(context);
	let mut tasks = task::JoinSet::new();
	for file in files {
		let context = context.clone();
		tasks.spawn(async move {
			if let Err(err) = fetch_file(&file, context).await {
				eprintln!("{} {}", "error:".red(), err);
			};
		});
	}

	while !tasks.is_empty() {
		// `tasks` is not empty, and must return a result
		tasks.join_next().await.unwrap()?;
	}

	Ok(())
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn parse_tagged_file_name() {
		let (file_path, url) = parse_file_name("Cargo.toml").unwrap();
		assert_eq!(file_path, "Cargo.toml");
		assert_eq!(url, BASE.join("Cargo.toml").unwrap());

		let (file_path, url) = parse_file_name("Cargo.toml@rust").unwrap();
		assert_eq!(file_path, "Cargo.toml");
		assert_eq!(url, BASE.join("@rust/Cargo.toml").unwrap());
	}

	#[test]
	fn parse_template_replacements() {
		let file_content = "Hi, my name is {{ name }}!";
		let captures = TEMPLATE_VARIABLE.captures(&file_content).unwrap();
		assert_eq!(&captures[1], "name");
		assert_eq!(captures.get(0).unwrap().range(), 15..25);

		// No spaces is fine
		let file_content = "Hi, my name is {{name}}!";
		let captures = TEMPLATE_VARIABLE.captures(&file_content).unwrap();
		assert_eq!(&captures[1], "name");
		assert_eq!(captures.get(0).unwrap().range(), 15..23);

		// Multiple spaces is fine
		let file_content = "Hi, my name is {{  name  }}!";
		let captures = TEMPLATE_VARIABLE.captures(&file_content).unwrap();
		assert_eq!(&captures[1], "name");
		assert_eq!(captures.get(0).unwrap().range(), 15..27);

		// `.` and `:` are fine
		let file_content = "Hi, my name is {{ github.username }}!";
		let captures = TEMPLATE_VARIABLE.captures(&file_content).unwrap();
		assert_eq!(&captures[1], "github.username");
		assert_eq!(captures.get(0).unwrap().range(), 15..36);

		// Numbers are fine, except at the start
		let file_content = "Hi, my name is {{ 0a.1b }}!";
		assert!(TEMPLATE_VARIABLE.captures(&file_content).is_none());
		let file_content = "Hi, my name is {{ a0.1b }}!";
		assert!(TEMPLATE_VARIABLE.captures(&file_content).is_none());
		let file_content = "Hi, my name is {{ a0.b1 }}!";
		let captures = TEMPLATE_VARIABLE.captures(&file_content).unwrap();
		assert_eq!(&captures[1], "a0.b1");
		assert_eq!(captures.get(0).unwrap().range(), 15..26);
		let file_content = "Hi, my name is {{ a0.1 }}!";
		let captures = TEMPLATE_VARIABLE.captures(&file_content).unwrap();
		assert_eq!(&captures[1], "a0.1");
		assert_eq!(captures.get(0).unwrap().range(), 15..25);
		let file_content = "Hi, my name is {{ 0.1 }}!";
		assert!(TEMPLATE_VARIABLE.captures(&file_content).is_none());

		// New lines, tabs, etc., are not fine
		let file_content = r#"
			Hi, my name is {{
				name
			}}!
		"#;
		assert!(TEMPLATE_VARIABLE.captures(&file_content).is_none());
	}
}
