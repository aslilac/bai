use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::process::Command;
use std::sync::LazyLock;

mod setup;

static EXE: LazyLock<PathBuf> = LazyLock::new(|| {
	Path::new("./build/release/bai")
		.canonicalize()
		.expect("unable to canonicalize path")
});

static DEFAULT_DEFINES: &[&str] = &[
	"-d",
	"git.branch=trunk",
	"-d",
	"author.name=James Baxter",
	"-d",
	"author.email=jamesbaxter@hey.com",
	"-d",
	"date.year=2112",
	"-d",
	"github.username=jamesbaxter",
];

#[test]
fn new_gleam_project() {
	setup::before();
	const PATH: &str = "./tests/testdata/gleam_project/";

	// Might fail if directory doesn't exist, but that's fine.
	_ = fs::remove_dir_all(PATH);
	fs::create_dir(PATH).expect("failed to create working directory");

	let result = Command::new(&*EXE)
		.current_dir(PATH)
		.args(DEFAULT_DEFINES)
		.arg("/gleam")
		.output()
		.unwrap();
	assert!(result.status.success());
	let stdout = String::from_utf8_lossy(&result.stdout);

	assert_eq!(stdout, "");
}
