//! Integration tests that run the real `gozo` binary against throwaway Go
//! projects. They are skipped when `go` is not installed.

use std::path::{Path, PathBuf};
use std::process::{Command, Output};

fn go_available() -> bool {
    Command::new("go")
        .arg("version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

fn gozo() -> Command {
    let mut c = Command::new(env!("CARGO_BIN_EXE_gozo"));
    c.env("NO_COLOR", "1").env_remove("GOZO_PROJECT");
    c
}

/// A single-module Go project with one main package under cmd/.
fn sample_module(dir: &Path, module: &str, cmd: &str) {
    std::fs::create_dir_all(dir.join("cmd").join(cmd)).unwrap();
    let ok = Command::new("go")
        .args(["mod", "init", module])
        .current_dir(dir)
        .output()
        .unwrap()
        .status
        .success();
    assert!(ok, "go mod init failed");
    std::fs::write(
        dir.join("cmd").join(cmd).join("main.go"),
        "package main\n\nimport \"fmt\"\n\nfunc main() { fmt.Println(\"hi\") }\n",
    )
    .unwrap();
}

fn tempdir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("gozo-it-{}-{}", name, std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn stdout(o: &Output) -> String {
    String::from_utf8_lossy(&o.stdout).into_owned()
}

fn stderr(o: &Output) -> String {
    String::from_utf8_lossy(&o.stderr).into_owned()
}

#[test]
fn help_lists_every_command() {
    let out = gozo().arg("--help").output().unwrap();
    assert!(out.status.success());
    let text = stdout(&out);
    for cmd in [
        "init", "link", "dev", "env", "doctor", "check", "test", "build", "generate", "run",
        "deploy", "status", "logs", "rollback", "tool", "update", "api",
    ] {
        assert!(
            text.contains(&format!("\n  {cmd}")),
            "help is missing `{cmd}`:\n{text}"
        );
    }
}

#[test]
fn doctor_reports_healthy_module_as_json() {
    if !go_available() {
        eprintln!("skipping: go not installed");
        return;
    }
    let dir = tempdir("doctor");
    sample_module(&dir, "example.com/demo", "demo");

    let out = gozo()
        .args(["doctor", "--offline", "--json", "-C"])
        .arg(&dir)
        .output()
        .unwrap();
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    let doc: serde_json::Value =
        serde_json::from_str(&stdout(&out)).expect("stdout is one JSON document");
    assert_eq!(doc["schema"], "gozo.doctor/v1");
    assert_eq!(doc["project"]["name"], "demo");
    assert_eq!(doc["project"]["kind"], "module");
    assert_eq!(doc["summary"]["fail"], 0);
    let ids: Vec<&str> = doc["findings"]
        .as_array()
        .unwrap()
        .iter()
        .map(|f| f["id"].as_str().unwrap())
        .collect();
    assert!(ids.contains(&"go.toolchain"));
    assert!(ids.contains(&"go.directive"));
}

#[test]
fn doctor_fails_on_replace_outside_project() {
    if !go_available() {
        eprintln!("skipping: go not installed");
        return;
    }
    let dir = tempdir("doctor-bad");
    sample_module(&dir, "example.com/bad", "bad");
    let gomod = dir.join("go.mod");
    let mut text = std::fs::read_to_string(&gomod).unwrap();
    text.push_str(
        "\nrequire github.com/google/uuid v1.6.0\n\nreplace github.com/google/uuid => ../outside\n",
    );
    std::fs::write(&gomod, text).unwrap();

    let out = gozo()
        .args(["doctor", "--offline", "--json", "-C"])
        .arg(&dir)
        .output()
        .unwrap();
    assert_eq!(
        out.status.code(),
        Some(1),
        "expected exit 1, stderr: {}",
        stderr(&out)
    );
    let doc: serde_json::Value = serde_json::from_str(&stdout(&out)).unwrap();
    let replace = doc["findings"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["id"] == "go.replace.local")
        .expect("replace finding present");
    assert_eq!(replace["status"], "fail");
}

#[test]
fn outside_a_project_errors_as_json() {
    let dir = tempdir("nothing");
    let out = gozo()
        .args(["doctor", "--json", "-C"])
        .arg(&dir)
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
    let doc: serde_json::Value = serde_json::from_str(&stdout(&out)).unwrap();
    assert_eq!(doc["schema"], "gozo.error/v1");
    assert!(doc["error"].as_str().unwrap().contains("no Go project"));
}

#[test]
fn init_link_env_api_round_trip() {
    if !go_available() {
        eprintln!("skipping: go not installed");
        return;
    }
    let dir = tempdir("workflow");

    let out = gozo()
        .args([
            "init",
            "--yes",
            "--module",
            "example.com/shop",
            "--cmd",
            "shop",
            "--json",
        ])
        .arg(&dir)
        .output()
        .unwrap();
    assert!(out.status.success(), "init failed: {}", stderr(&out));
    let doc: serde_json::Value = serde_json::from_str(&stdout(&out)).unwrap();
    assert_eq!(doc["schema"], "gozo.init/v1");
    assert!(dir.join("gozo.toml").is_file());
    assert!(dir.join("cmd/shop/main.go").is_file());

    let out = gozo()
        .args(["link", "--yes", "--target", "docker", "--json", "-C"])
        .arg(&dir)
        .output()
        .unwrap();
    assert!(out.status.success(), "link failed: {}", stderr(&out));
    assert!(dir.join(".gozo/project.json").is_file());

    let out = gozo()
        .args([
            "env",
            "add",
            "DATABASE_URL",
            "development",
            "--value",
            "postgres://x",
            "--json",
            "-C",
        ])
        .arg(&dir)
        .output()
        .unwrap();
    assert!(out.status.success(), "env add failed: {}", stderr(&out));

    let out = gozo()
        .args(["env", "ls", "development", "--show-values", "--json", "-C"])
        .arg(&dir)
        .output()
        .unwrap();
    let doc: serde_json::Value = serde_json::from_str(&stdout(&out)).unwrap();
    assert_eq!(doc["schema"], "gozo.env/v1");
    assert_eq!(doc["variables"][0]["key"], "DATABASE_URL");
    assert_eq!(doc["variables"][0]["value"], "postgres://x");

    let out = gozo()
        .args(["env", "pull", "--yes", "-C"])
        .arg(&dir)
        .output()
        .unwrap();
    assert!(out.status.success(), "env pull failed: {}", stderr(&out));
    let env_file = std::fs::read_to_string(dir.join(".env.local")).unwrap();
    assert!(env_file.contains("DATABASE_URL=postgres://x"), "{env_file}");
    assert!(
        stdout(&out).trim().ends_with(".env.local"),
        "stdout is the file path"
    );

    let out = gozo()
        .args(["api", "project", "-C"])
        .arg(&dir)
        .output()
        .unwrap();
    let doc: serde_json::Value = serde_json::from_str(&stdout(&out)).unwrap();
    assert_eq!(doc["schema"], "gozo.api/v1");
    assert_eq!(doc["data"]["name"], "shop");
    assert_eq!(doc["data"]["linked"]["target"], "docker");

    let out = gozo()
        .args(["check", "--json", "-C"])
        .arg(&dir)
        .output()
        .unwrap();
    assert!(out.status.success(), "check failed: {}", stderr(&out));
    let doc: serde_json::Value = serde_json::from_str(&stdout(&out)).unwrap();
    assert_eq!(doc["schema"], "gozo.check/v1");
    assert_eq!(doc["summary"]["fail"], 0);

    let out = gozo()
        .args(["build", "--version", "test", "--json", "-C"])
        .arg(&dir)
        .output()
        .unwrap();
    assert!(out.status.success(), "build failed: {}", stderr(&out));
    let doc: serde_json::Value = serde_json::from_str(&stdout(&out)).unwrap();
    assert_eq!(doc["artifacts"][0]["name"], "shop");
    assert!(dir.join("dist/shop").is_file() || dir.join("dist/shop.exe").is_file());
}

#[test]
fn bare_gozo_non_interactive_prints_detection_block() {
    if !go_available() {
        eprintln!("skipping: go not installed");
        return;
    }
    let dir = tempdir("home");
    sample_module(&dir, "example.com/home", "home");
    let out = gozo().args(["--json", "-C"]).arg(&dir).output().unwrap();
    assert!(out.status.success(), "stderr: {}", stderr(&out));
    let doc: serde_json::Value = serde_json::from_str(&stdout(&out)).unwrap();
    assert_eq!(doc["schema"], "gozo.project/v1");
    assert_eq!(doc["name"], "home");
    assert_eq!(doc["detected"], true);
}
