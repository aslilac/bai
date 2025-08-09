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

#[test]
fn hello() {
	setup::before();

	let result = Command::new(EXE).output().unwrap();
	assert!(result.status.success());
	let stdout = String::from_utf8_lossy(&result.stdout);

	assert!(stdout.contains("hello, computer!"));
}
