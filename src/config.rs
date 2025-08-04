use etcetera::BaseStrategy;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
	#[serde(default, skip_serializing_if = "HashMap::is_empty")]
	pub context: HashMap<String, String>,
}

impl Config {
	pub fn file_path() -> anyhow::Result<PathBuf> {
		Ok(etcetera::choose_base_strategy()?.config_dir().join("bai.toml"))
	}

	pub fn init() -> anyhow::Result<Self> {
		let mut buf = String::with_capacity(100);
		let stdin = io::stdin();
		let mut stdout = io::stdout();
		write!(stdout, "github.username? ")?;
		stdout.flush().unwrap();
		stdin.read_line(&mut buf).unwrap();
		let github_username = buf.trim().to_string();

		if !github_username.is_empty() {
			Self::set_context([("github.username", &github_username)])
				.expect("failed to update config");
		}

		Ok(Config {
			context: HashMap::from([(
				"github.username".to_string(),
				github_username,
			)]),
		})
	}

	pub fn load() -> anyhow::Result<Self> {
		let config_file = Self::file_path()?;
		if !config_file.exists() {
			return Self::init();
		}

		let config =
			toml::from_str(&fs::read_to_string(&config_file).unwrap_or_default())?;
		Ok(config)
	}

	pub fn set_context<I, K, V>(values: I) -> anyhow::Result<()>
	where
		I: IntoIterator<Item = (K, V)>,
		K: AsRef<str>,
		V: AsRef<str>,
	{
		let config_file = Self::file_path()?;

		let content = &fs::read_to_string(&config_file).unwrap_or_default();
		let mut config = content.parse::<toml_edit::Document>()?;
		if !config.contains_table("context") {
			config["context"] = toml_edit::Item::Table(toml_edit::Table::new());
		}

		for (key, value) in values.into_iter() {
			config["context"][key.as_ref()] = toml_edit::value(value.as_ref());
		}

		let parent_exists = config_file.parent().map(Path::exists).unwrap_or(true);
		if !parent_exists {
			fs::create_dir_all(config_file.parent().unwrap())?;
		}

		config["context"].as_table_mut().unwrap().sort_values();
		fs::write(&config_file, config.to_string())?;
		Ok(())
	}
}
