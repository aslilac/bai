use regex::Captures;
use regex::Regex;

pub fn for_each<T, F>(regex: &Regex, text: String, each: F) -> String
where
	T: AsRef<str>,
	F: Fn(&Captures<'_>) -> Option<T>,
{
	let mut replacements = regex.captures_iter(&text).peekable();

	// Don't allocate anything if there are no replacements, just use the unaltered content
	if replacements.peek().is_none() {
		text
	} else {
		let mut processed_content = String::with_capacity(text.len());
		let mut i = 0;
		for captures in replacements {
			let range = captures.get(0).unwrap().range();
			let replacement = each(&captures);
			match replacement {
				Some(replacement) => {
					processed_content.push_str(&text[i..range.start]);
					processed_content.push_str(replacement.as_ref());
				}
				None => {
					processed_content.push_str(&text[i..range.end]);
				}
			}
			i = range.end;
		}

		processed_content.push_str(&text[i..]);
		processed_content
	}
}
