use anyhow::anyhow;
use colored::Colorize;
use once_cell::sync::Lazy;
use regex::Regex;
use std::collections::BTreeSet;
use std::collections::HashMap;
use std::process::exit;

use crate::IDENT;
use crate::config::Config;
use crate::groups::expand_group;

static VARIABLE_NAME: Lazy<Regex> =
	Lazy::new(|| Regex::new(&format!("^{}$", *IDENT)).unwrap());

#[derive(Clone, Debug)]
pub struct Options {
	pub files: BTreeSet<String>,
	pub context: HashMap<String, String>,
	pub aliases: Vec<(String, String)>,
}

impl<S, const N: usize> TryFrom<&[S; N]> for Options
where
	S: AsRef<str>,
{
	type Error = anyhow::Error;

	fn try_from(args: &[S; N]) -> Result<Self, Self::Error> {
		Options::try_from(&args[..])
	}
}

fn help() {
	println!("{}", include_str!("./help.txt"));
	exit(0);
}

impl<S> TryFrom<&[S]> for Options
where
	S: AsRef<str>,
{
	type Error = anyhow::Error;

	fn try_from(args: &[S]) -> Result<Self, Self::Error> {
		if args.is_empty() {
			help();
		}

		if args.len() == 1 {
			match args[0].as_ref() {
				"-h" | "-help" | "--help" | "-?" | "help" => help(),
				"-v" | "-V" | "-version" | "--version" | "version" => {
					println!(
						"{} {}",
						env!("CARGO_PKG_NAME").bright_magenta().bold(),
						env!("CARGO_PKG_VERSION").bold()
					);
					exit(0);
				}
				"-config" | "--config" | "-get-config-path" | "--get-config-path" => {
					println!("{}", Config::file_path()?.display());
					exit(0);
				}
				_ => (),
			}
		}

		if matches!(args[0].as_ref(), "-set" | "--set") {
			let definitions = args.iter().skip(1).filter_map(|definition| {
				let definition = definition.as_ref();
				let Some((key, value)) = definition.split_once('=') else {
					eprintln!(
						"{} invalid definition \"{}\", must contain a \"=\" to separate the name and value",
						"warning:".yellow(),
						definition
					);
					return None;
				};
				let Some(_) = VARIABLE_NAME.find_at(key, 0) else {
					eprintln!("{} key \"{}\" is invalid", "warning:".yellow(), key);
					return None;
				};
				Some((key, value))
			});

			Config::set_context(definitions)?;
			exit(0);
		}

		let mut args = args.iter();
		let mut files = Vec::new();
		let mut context = HashMap::new();
		let mut aliases = vec![];

		while let Some(arg) = args.next() {
			let arg = arg.as_ref();
			match arg {
				"-d" | "-D" | "-define" | "--define" => {
					let definition = args
						.next()
						.ok_or_else(|| anyhow!("expected a definition after {}", arg))?
						.as_ref();

					let (key, value) = definition.split_once('=').ok_or_else(|| {
						anyhow!(
							"invalid definition \"{}\", must contain a \"=\" to separate the name and value",
							definition
						)
					})?;
					VARIABLE_NAME
						.find_at(key, 0)
						.ok_or_else(|| anyhow!("key \"{}\" is invalid", key))?;
					context.insert(key.to_string(), value.to_string());
				}
				"-use" | "--use" | "-alias" | "--alias" | "-A" => {
					let alias = args
						.next()
						.ok_or_else(|| anyhow!("expected an alias after {}", arg))?
						.as_ref();

					let (alias, canonical_name) = alias.split_once('=').ok_or_else(|| {
						anyhow!(
							"invalid alias \"{}\", must contain a \"=\" to separate the alias and canonical name",
							alias
						)
					})?;
					VARIABLE_NAME
						.find_at(alias, 0)
						.ok_or_else(|| anyhow!("alias \"{}\" is invalid", alias))?;
					VARIABLE_NAME.find_at(canonical_name, 0).ok_or_else(|| {
						anyhow!("canonical name \"{}\" is invalid", canonical_name)
					})?;
					aliases.push((alias.to_string(), canonical_name.to_string()));
				}
				_ => {
					if arg.len() >= 2 && arg.starts_with('-') {
						return Err(anyhow!("unrecognized option: {}", arg));
					} else {
						files.push(arg);
					};
				}
			}
		}

		let files = files
			.into_iter()
			.flat_map(|arg| {
				if arg.starts_with("/") {
					return expand_group(arg)
						.inspect_err(|err| eprintln!("{} {}", "warning:".yellow(), err))
						.unwrap_or_default()
						.iter()
						.map(|s| s.to_string())
						.collect();
				}

				vec![arg.to_string()]
			})
			.collect();

		Ok(Options { files, context, aliases })
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn groups() {
		let options = Options::try_from(&["/gleam"]).unwrap();
		assert!(options.files.contains("gleam.toml"));
	}

	#[test]
	fn parse_identifier() {
		assert!(VARIABLE_NAME.find_at("a", 0).is_some());
		assert!(VARIABLE_NAME.find_at("a.b", 0).is_some());
		assert!(VARIABLE_NAME.find_at("a.", 0).is_none());
		assert!(VARIABLE_NAME.find_at("hello.friend", 0).is_some());
		assert!(VARIABLE_NAME.find_at("hello.friend-name", 0).is_some());
		assert!(VARIABLE_NAME.find_at("hello.friend_name", 0).is_some());
		assert!(VARIABLE_NAME.find_at("hello.friend-", 0).is_none());
		assert!(VARIABLE_NAME.find_at("hello.friend_", 0).is_none());
		assert!(VARIABLE_NAME.find_at("hello.friend_name1", 0).is_some());
		assert!(VARIABLE_NAME.find_at("hello.0", 0).is_some());
		assert!(VARIABLE_NAME.find_at("hello.0a", 0).is_none());
		assert!(VARIABLE_NAME.find_at("hello.01", 0).is_none());
		assert!(VARIABLE_NAME.find_at("hello.10", 0).is_some());
		assert!(VARIABLE_NAME.find_at("0.1", 0).is_none());
		assert!(VARIABLE_NAME.find_at("0a.1", 0).is_none());
		assert!(VARIABLE_NAME.find_at("a0.1", 0).is_some());
	}
}
