use anyhow::anyhow;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::fs;
use std::io;
use std::io::Write;
use std::path::Path;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Config {
	#[serde(default, skip_serializing_if = "HashMap::is_empty")]
	pub context: HashMap<String, String>,
}

impl Config {
	pub fn init() -> anyhow::Result<Self> {
		let mut buf = String::with_capacity(100);
		let stdin = io::stdin();
		write!(io::stdout(), "github.username? ")?;
		io::stdout().flush().unwrap();
		stdin.read_line(&mut buf).unwrap();
		let github_username = buf.trim().to_string();

		if !github_username.is_empty() {
			Self::set_context([("github.username", &github_username)])
				.expect("failed to update config");
		}

		Ok(Config {
			context: HashMap::from([("github.username".to_string(), github_username)]),
		})
	}

	pub fn load() -> anyhow::Result<Self> {
		// let xdg_base = xdg::BaseDirectories::with_prefix("okie")?;
		let xdg_base = xdg::BaseDirectories::new()?;
		let config_file = xdg_base.get_config_file(Path::new("okie.toml"));

		if !config_file.exists() {
			return Self::init();
		}

		let config = toml::from_str(&fs::read_to_string(&config_file).unwrap_or_default())?;
		Ok(config)
	}

	pub fn set_context<I, K, V>(values: I) -> anyhow::Result<()>
	where
		I: IntoIterator<Item = (K, V)>,
		K: AsRef<str>,
		V: AsRef<str>,
	{
		// let xdg_base = xdg::BaseDirectories::with_prefix("okie")?;
		let xdg_base = xdg::BaseDirectories::new()?;
		let config_file = xdg_base
			.place_config_file(Path::new("okie.toml"))
			.map_err(|err| anyhow!(err))?;

		let content = &fs::read_to_string(&config_file).unwrap_or_default();
		let mut config = content.parse::<toml_edit::Document>()?;
		// let mut context = match config["context"] {
		//   toml_edit::Item::None => toml_edit::Table::new(),
		//   toml_edit::Item::Table(table) => table,
		// }.clone().into_table().unwrap();

		if !config.contains_table("context") {
			config["context"] = toml_edit::Item::Table(toml_edit::Table::new());
		}

		for (key, value) in values.into_iter() {
			config["context"][key.as_ref()] = toml_edit::value(value.as_ref());
			// config["context"].as_table_mut().unwrap().
		}

		// config.sort_values();
		// config.insert("context", toml_edit::Item::Table(context));
		config["context"].as_table_mut().unwrap().sort_values();
		fs::write(&config_file, config.to_string())?;
		Ok(())
	}
}
