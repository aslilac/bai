#[derive(Clone, Debug)]
pub struct Options {
	pub files: Vec<String>,
}

impl<S> FromIterator<S> for Options
where
	S: AsRef<str>,
{
	fn from_iter<I: IntoIterator<Item = S>>(args: I) -> Self {
		let args = args.into_iter();
		let files = args.map(|arg| arg.as_ref().to_string()).collect();
		Options { files }
	}
}
