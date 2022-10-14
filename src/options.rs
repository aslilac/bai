use colored::Colorize;
use std::collections::BTreeSet;

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
		let files = args
			.map(|arg| {
				if arg.as_ref().starts_with("/") {
					return expand_group(arg)
						.unwrap_or_else(|err| {
							println!("{} {}", Colorize::yellow("warning:"), err);
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
