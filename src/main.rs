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

mod config;
mod groups;
mod options;
mod regext;
use config::Config;
use options::Options;

static BASE: Lazy<reqwest::Url> = Lazy::new(|| {
	reqwest::Url::parse("https://raw.githubusercontent.com/aslilac/bai/main/static/")
		.expect("invalid base URL")
});

static IDENT: Lazy<&str> = Lazy::new(|| include_str!("./ident.pcre").trim_end());
static TEMPLATE_VARIABLE: Lazy<Regex> =
	Lazy::new(|| Regex::new(&format!("\\{{\\{{ *{} *\\}}\\}}", *IDENT)).unwrap());
static PATH_TEMPLATE_VARIABLE: Lazy<Regex> =
	Lazy::new(|| Regex::new(&format!("\\$\\${}\\$\\$", *IDENT)).unwrap());

type Context = HashMap<String, String>;

fn parse_file_name(file: &str) -> anyhow::Result<(&str, reqwest::Url)> {
	let (file_path, tag) = file
		.rsplit_once("@")
		.map(|(file_path, tag)| (file_path, Some(tag.to_ascii_lowercase())))
		.unwrap_or((file, None));

	let mut base = BASE.clone();
	if let Some(tag) = tag {
		base = base.join(&format!("@{tag}/"))?;
	}

	Ok((file_path, base.join(file_path)?))
}

async fn fetch_file<C>(file: &str, ctx: C) -> anyhow::Result<()>
where
	C: AsRef<Context>,
{
	let ctx = ctx.as_ref();
	let (file_path, url) = parse_file_name(file)?;

	// Fetch file
	let file_content = reqwest::get(url).await?.error_for_status()?.text().await?;

	let each = |captures: &regex::Captures| ctx.get(&captures[1]);
	// Fill in template variables
	let file_content = regext::for_each(&TEMPLATE_VARIABLE, file_content, each);
	let file_path = regext::for_each(&PATH_TEMPLATE_VARIABLE, file_path.to_string(), each);

	// Create parent directories as necessary
	if let Some(parent) = Path::new(&file_path).parent()
		&& !parent.exists()
	{
		fs::create_dir_all(parent)?;
	}
	fs::write(file_path, file_content)?;

	Ok(())
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
	let Options {
		files,
		mut context,
		aliases,
	} = Options::try_from(&*env::args().skip(1).collect::<Vec<_>>())?;

	let mut config = Config::load()?;
	// Copy context variables defined as arguments _over_ context variables loaded
	// from the config file.
	config.context.extend(context);
	context = config.context;

	if !context.contains_key("name") {
		if let Some(dir) = env::current_dir()
			.ok()
			.and_then(|dir| dir.file_name().map(|name| name.to_os_string()))
		{
			context.insert("name".to_string(), dir.to_string_lossy().to_string());
		} else {
			eprintln!(
				"{} name is unset, but is used by many templates",
				"warning:".yellow(),
			);
			eprintln!(
				"{} try running:\n    bai [files...] -define \"name=my_project\"",
				"fix:".green(),
			);
		}
	};

	if !context.contains_key("git.branch") {
		let output = Command::new("git")
			.args(["config", "init.defaultBranch"])
			.output();
		if let Ok(output) = output
			&& output.status.success()
		{
			let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
			context.insert("git.branch".to_string(), stdout);
		} else {
			// The command might fail if a value hasn't been set, but we should just
			// gracefully fall back to Git's default.
			context.insert("git.branch".to_string(), "master".to_string());
		}
	};

	if !context.contains_key("author.name") {
		let output = Command::new("git").args(["config", "user.name"]).output();

		if let Ok(output) = output
			&& output.status.success()
		{
			// Ouch. Two allocations in one line.
			let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
			context.insert("author.name".to_string(), stdout);
		} else {
			eprintln!(
				"{} author.name is unset, but is used by many templates",
				"warning:".yellow()
			);
			eprintln!(
				"{} author.name can be set by running\n    bai -set \"author.name=James Baxter\"",
				"fix:".green()
			);
			eprintln!(
				"{} author.name can also be inferred from git\n    git config --global user.name \"James Baxter\"",
				"fix:".green()
			);
		}
	};

	if !context.contains_key("author.email") {
		let output = Command::new("git").args(["config", "user.email"]).output();

		if let Ok(output) = output
			&& output.status.success()
		{
			// Ouch. Two allocations in one line.
			let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
			context.insert("author.email".to_string(), stdout);
		} else {
			eprintln!(
				"{} author.email is unset, but is used by many templates",
				"warning:".yellow()
			);
			eprintln!(
				"{} author.email can be set by running\n    bai -set \"author.email=jamesbaxter@hey.com\"",
				"fix:".green()
			);
			eprintln!(
				"{} author.email can also be inferred from git\n    git config --global user.email \"jamesbaxter@hey.com\"",
				"fix:".green()
			);
		}
	};

	if !context.contains_key("date.year") {
		context.insert(
			"date.year".to_string(),
			chrono::Local::now().year().to_string(),
		);
	}

	if context.contains_key("github.username") && !context.contains_key("github.owner") {
		context.insert(
			"github.owner".to_string(),
			context["github.username"].to_string(),
		);
	}

	for (alias, canonical_name) in aliases {
		if context.contains_key(&canonical_name) {
			if !context.contains_key(&alias) {
				context.insert(alias, context[&canonical_name].clone());
			} else {
				eprintln!(
					"{0} {1} was aliased to {2}, but {1} is already set",
					"warning:".yellow(),
					alias,
					canonical_name,
				);
			}
		} else {
			eprintln!(
				"{0} {1} was aliased to {2}, but {2} is not set",
				"warning:".yellow(),
				alias,
				canonical_name,
			);
		}
	}

	if context.contains_key("author.name")
		&& !context.contains_key("licence.owner")
		&& !context.contains_key("license.owner")
	{
		context.insert(
			"licence.owner".to_string(),
			context["author.name"].to_string(),
		);
		context.insert(
			"license.owner".to_string(),
			context["author.name"].to_string(),
		);
	}

	if context.contains_key("license.owner") && !context.contains_key("licence.owner") {
		context.insert(
			"licence.owner".to_string(),
			context["license.owner"].to_string(),
		);
	}

	if context.contains_key("licence.owner") && !context.contains_key("license.owner") {
		context.insert(
			"license.owner".to_string(),
			context["licence.owner"].to_string(),
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
		let captures = TEMPLATE_VARIABLE.captures(file_content).unwrap();
		assert_eq!(&captures[1], "name");
		assert_eq!(captures.get(0).unwrap().range(), 15..25);

		// No spaces is fine
		let file_content = "Hi, my name is {{name}}!";
		let captures = TEMPLATE_VARIABLE.captures(file_content).unwrap();
		assert_eq!(&captures[1], "name");
		assert_eq!(captures.get(0).unwrap().range(), 15..23);

		// Multiple spaces is fine
		let file_content = "Hi, my name is {{  name  }}!";
		let captures = TEMPLATE_VARIABLE.captures(file_content).unwrap();
		assert_eq!(&captures[1], "name");
		assert_eq!(captures.get(0).unwrap().range(), 15..27);

		// `.` and `:` are fine
		let file_content = "Hi, my name is {{ github.username }}!";
		let captures = TEMPLATE_VARIABLE.captures(file_content).unwrap();
		assert_eq!(&captures[1], "github.username");
		assert_eq!(captures.get(0).unwrap().range(), 15..36);

		// Numbers are fine, except at the start
		let file_content = "Hi, my name is {{ 0a.1b }}!";
		assert!(TEMPLATE_VARIABLE.captures(file_content).is_none());
		let file_content = "Hi, my name is {{ a0.1b }}!";
		assert!(TEMPLATE_VARIABLE.captures(file_content).is_none());
		let file_content = "Hi, my name is {{ a0.b1 }}!";
		let captures = TEMPLATE_VARIABLE.captures(file_content).unwrap();
		assert_eq!(&captures[1], "a0.b1");
		assert_eq!(captures.get(0).unwrap().range(), 15..26);
		let file_content = "Hi, my name is {{ a0.1 }}!";
		let captures = TEMPLATE_VARIABLE.captures(file_content).unwrap();
		assert_eq!(&captures[1], "a0.1");
		assert_eq!(captures.get(0).unwrap().range(), 15..25);
		let file_content = "Hi, my name is {{ 0.1 }}!";
		assert!(TEMPLATE_VARIABLE.captures(file_content).is_none());

		// New lines, tabs, etc., are not fine
		let file_content = r#"
			Hi, my name is {{
				name
			}}!
		"#;
		assert!(TEMPLATE_VARIABLE.captures(file_content).is_none());
	}
}
