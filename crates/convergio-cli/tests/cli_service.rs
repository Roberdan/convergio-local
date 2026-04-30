//! Service command smoke tests.

use assert_cmd::Command;
use predicates::prelude::*;
use std::{env, ffi::OsString, fs, path::Path};

fn cvg() -> Command {
    Command::cargo_bin("cvg").expect("cvg binary built")
}

fn path_with_bin_dir(bin_dir: &Path) -> OsString {
    let mut paths = vec![bin_dir.to_path_buf()];
    if let Some(path) = env::var_os("PATH") {
        paths.extend(env::split_paths(&path));
    }
    env::join_paths(paths).expect("valid PATH")
}

#[test]
fn service_install_writes_user_service_file() {
    let home = tempfile::tempdir().expect("temp home");
    let bin_dir = tempfile::tempdir().expect("temp bin");
    let convergio = bin_dir.path().join("convergio");
    fs::write(&convergio, "#!/bin/sh\nexit 0\n").expect("fake convergio");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(&convergio)
            .expect("fake convergio metadata")
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&convergio, permissions).expect("fake convergio executable");
    }

    cvg()
        .env("HOME", home.path())
        .env("PATH", path_with_bin_dir(bin_dir.path()))
        .args(["service", "install", "--force"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Service file written"));

    let macos = home
        .path()
        .join("Library/LaunchAgents/com.convergio.v3.plist");
    let linux = home.path().join(".config/systemd/user/convergio.service");
    assert!(macos.is_file() || linux.is_file());
}
