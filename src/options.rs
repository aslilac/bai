use colored::Colorize;
use std::collections::BTreeSet;
use std::process::exit;

use crate::groups::expand_group;

#[derive(Clone, Debug)]
pub struct Options {
	pub files: BTreeSet<String>,
}

impl<S> FromIterator<S> for Options
where
	S: AsRef<str>,
{
	fn from_iter<I: IntoIterator<Item = S>>(args: I) -> Self {
		let args = args.into_iter();

		let (flags, files) = args.partition::<Vec<_>, _>(|arg| {
			let arg = arg.as_ref();
			(arg.len() > 2 && arg.starts_with('-')) || (arg.len() > 3 && arg.starts_with("--"))
		});

		for flag in flags {
			let flag = flag.as_ref();
			match flag {
				"-v" | "-V" | "--version" => {
					println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
					exit(0);
				}
				_ => {
					eprintln!(
						"{} {}",
						"error:".red(),
						format!("unrecognized flag: {}", flag)
					);
					exit(1);
				}
			}
		}

		let files = files
			.iter()
			.map(|arg| {
				if arg.as_ref().starts_with("/") {
					// TODO: use `inspect_err` and `unwrap_or_default` when that stabilizes
					// https://github.com/rust-lang/rust/issues/91345
					return expand_group(arg)
						.unwrap_or_else(|err| {
							eprintln!("{} {}", "warning:".yellow(), err);
							Default::default()
						})
						.iter()
						.map(|s| s.to_string())
						.collect();
				}

				vec![arg.as_ref().to_string()]
			})
			.flatten()
			.collect();

		Options { files }
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn groups() {
		let options = ["/gleam"].into_iter().collect::<Options>();
		assert!(options.files.contains("gleam.toml"));
	}
}
