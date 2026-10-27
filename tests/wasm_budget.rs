//! The WASM size gate must be a real gate: it has to fail on an over-budget
//! bundle and on a missing bundle, not just print numbers. These tests drive
//! the same script CI runs, using a temporary `dist/` and a fake artifact.

#[cfg(unix)]
mod unix {
  use std::fs;
  use std::path::{Path, PathBuf};
  use std::process::{Command, Output};
  use std::time::{SystemTime, UNIX_EPOCH};

  fn script() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("scripts/check-wasm-size.sh")
  }

  fn temp_dist(name: &str) -> PathBuf {
    let nanos = SystemTime::now()
      .duration_since(UNIX_EPOCH)
      .expect("system clock after the epoch")
      .as_nanos();
    let dir = std::env::temp_dir().join(format!(
      "rust-blog-wasm-budget-{name}-{}-{nanos}",
      std::process::id()
    ));
    fs::create_dir_all(dir.join("dist")).expect("temp dist is creatable");
    dir
  }

  fn run(dir: &Path, budget_kib: &str) -> Output {
    Command::new("sh")
      .arg(script())
      .env("WASM_GZIP_BUDGET_KIB", budget_kib)
      .current_dir(dir)
      .output()
      .expect("the size script must be runnable")
  }

  #[test]
  fn fails_when_the_bundle_exceeds_the_budget() {
    let dir = temp_dist("over");
    fs::write(dir.join("dist/app_bg.wasm"), vec![0u8; 1024 * 1024]).expect("fake wasm is writable");

    let output = run(&dir, "0");
    assert!(
      !output.status.success(),
      "an over-budget bundle must fail the gate: {output:?}"
    );
    assert!(
      String::from_utf8_lossy(&output.stderr).contains("budget"),
      "the failure must name the budget: {output:?}"
    );

    fs::remove_dir_all(dir).expect("temp dir is removable");
  }

  #[test]
  fn passes_when_the_bundle_is_within_budget() {
    let dir = temp_dist("under");
    fs::write(dir.join("dist/app_bg.wasm"), vec![0u8; 1024 * 1024]).expect("fake wasm is writable");

    let output = run(&dir, "1024");
    assert!(
      output.status.success(),
      "a bundle within the budget must pass: {output:?}"
    );

    fs::remove_dir_all(dir).expect("temp dir is removable");
  }

  #[test]
  fn fails_when_no_wasm_artifact_exists() {
    let dir = temp_dist("missing");

    let output = run(&dir, "1024");
    assert!(
      !output.status.success(),
      "a missing wasm artifact must fail the gate: {output:?}"
    );
    assert!(
      String::from_utf8_lossy(&output.stderr).contains("no *_bg.wasm"),
      "the failure must explain the missing artifact: {output:?}"
    );

    fs::remove_dir_all(dir).expect("temp dir is removable");
  }
}
