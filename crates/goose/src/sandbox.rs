//! OS-level sandbox for commands spawned by agent tools (pleumcode).
//!
//! This is a *security boundary*, unlike `tool_permissions` / approval modes,
//! which are guardrails that `bash -c`, aliases or quoting can bypass. It is
//! deliberately not configurable at runtime: no env var, no config key.
//!
//! Policy (Codex-style split: the agent process itself stays unsandboxed so it
//! can reach the model API; only the commands it spawns are confined):
//! - writes only inside the workspace and the temp dir (`.git/hooks` and
//!   `.git/config` stay read-only: a hook is a later, unsandboxed host exec)
//! - no network at all
//! - credential dirs (`~/.ssh`, `~/.aws`, `~/.gnupg`, goose config) unreadable,
//!   since anything a command prints ends up in the model's context
//!
//! macOS only (Seatbelt via `sandbox-exec`). Every other OS fails closed:
//! `wrap` returns an error and the command is not run.
//! ponytail: Linux (bubblewrap) is the next backend; not implemented yet.

use std::io;
use std::path::{Path, PathBuf};
use tokio::process::Command;

#[cfg(target_os = "macos")]
const SANDBOX_EXEC: &str = "/usr/bin/sandbox-exec";

// Paths come in as `-D` params, never spliced into the profile text.
// Seatbelt matches canonical paths, so callers must pass realpaths (`/private/var`, not `/var`).
#[cfg(target_os = "macos")]
const PROFILE: &str = r#"(version 1)
(deny default)
(allow process-exec*)
(allow process-fork)
(allow signal (target self))
(allow sysctl-read)
(allow mach-lookup)
(allow ipc-posix-shm)
(allow file-read*)
(deny file-read*
  (subpath (string-append (param "HOME") "/.ssh"))
  (subpath (string-append (param "HOME") "/.aws"))
  (subpath (string-append (param "HOME") "/.gnupg"))
  (subpath (param "GOOSE_CONFIG")))
(allow file-write*
  (subpath (param "WS"))
  (subpath (param "TMP"))
  (literal "/dev/null")
  (literal "/dev/tty")
  (literal "/dev/dtracehelper"))
(deny file-write*
  (subpath (string-append (param "WS") "/.git/hooks"))
  (literal (string-append (param "WS") "/.git/config")))
"#;

/// Returns `cmd` re-targeted so it runs inside the sandbox, confined to `workspace`.
/// Program, args, env and cwd are preserved. Fails closed if the sandbox is unavailable.
pub fn wrap(cmd: Command, workspace: &Path) -> io::Result<Command> {
    #[cfg(target_os = "macos")]
    {
        let real = |p: &Path| std::fs::canonicalize(p);
        let ws = real(workspace)?;
        let tmp = real(&std::env::temp_dir())?;
        let home = std::env::var_os("HOME")
            .map(PathBuf::from)
            .and_then(|h| real(&h).ok())
            .ok_or_else(|| io::Error::other("sandbox: cannot resolve $HOME"))?;
        // Config dir may not exist yet; canonicalize the parent chain we can and fall back.
        let goose_cfg = crate::config::paths::Paths::config_dir();
        let goose_cfg = real(&goose_cfg).unwrap_or(goose_cfg);

        let std_cmd = cmd.as_std();
        let mut out = Command::new(SANDBOX_EXEC);
        out.arg("-p").arg(PROFILE);
        for (k, v) in [
            ("WS", &ws),
            ("TMP", &tmp),
            ("HOME", &home),
            ("GOOSE_CONFIG", &goose_cfg),
        ] {
            out.arg("-D").arg(format!("{k}={}", v.display()));
        }
        out.arg(std_cmd.get_program()).args(std_cmd.get_args());
        for (k, v) in std_cmd.get_envs() {
            match v {
                Some(v) => out.env(k, v),
                None => out.env_remove(k),
            };
        }
        if let Some(dir) = std_cmd.get_current_dir() {
            out.current_dir(dir);
        }
        Ok(out)
    }
    #[cfg(not(target_os = "macos"))]
    {
        let _ = (cmd, workspace);
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "sandbox: no OS sandbox backend for this platform; refusing to run command unconfined",
        ))
    }
}

/// In-process counterpart of the Seatbelt write policy, for tools that write files
/// inside the agent process (where `wrap` can't reach). Same rules: workspace + temp
/// dir only, `.git/hooks` and `.git/config` excluded. Keep in sync with `PROFILE`.
///
/// Symlinks are resolved before the check, and a not-yet-existing tail may not
/// contain `..`, so neither `link -> /etc` nor `new/../../x` can escape.
pub fn check_write(path: &Path, workspace: &Path) -> Result<(), String> {
    let deny = |why: &str| Err(format!("Refusing to write {}: {why}", path.display()));

    // Walk up to the deepest existing ancestor (symlink_metadata: a dangling symlink counts as existing).
    let mut existing = path.to_path_buf();
    let mut tail = Vec::new();
    while std::fs::symlink_metadata(&existing).is_err() {
        match existing.file_name() {
            Some(name) => tail.push(name.to_owned()),
            None => return deny("path contains `..` below a non-existent directory"),
        }
        if !existing.pop() {
            return deny("no existing ancestor");
        }
    }
    let mut target = match std::fs::canonicalize(&existing) {
        Ok(p) => p,
        Err(_) => return deny("cannot resolve path (dangling symlink?)"),
    };
    target.extend(tail.iter().rev());

    let ws =
        std::fs::canonicalize(workspace).map_err(|e| format!("sandbox: bad workspace: {e}"))?;
    let tmp = std::fs::canonicalize(std::env::temp_dir())
        .unwrap_or_else(|_| PathBuf::from("/nonexistent"));
    if !(target.starts_with(&ws) || target.starts_with(&tmp)) {
        return deny("outside the workspace");
    }
    if target.starts_with(ws.join(".git/hooks")) || target == ws.join(".git/config") {
        return deny("git hooks/config are read-only");
    }
    Ok(())
}

#[cfg(all(test, target_os = "macos"))]
mod tests {
    use super::*;
    use std::process::Output;

    async fn run(ws: &Path, script: &str) -> Output {
        let mut c = Command::new("/bin/sh");
        c.arg("-c").arg(script).current_dir(ws);
        wrap(c, ws).unwrap().output().await.unwrap()
    }

    fn ok(o: &Output) -> bool {
        o.status.success()
    }

    #[tokio::test]
    async fn confines_writes_network_and_secrets() {
        // Parent lives under $HOME, not the temp dir (temp is writable by policy),
        // so `..` from the workspace lands somewhere that must be denied.
        let dir = tempfile::Builder::new()
            .prefix("pleum-sbx-test")
            .tempdir_in(dirs::home_dir().unwrap())
            .unwrap();
        let ws = dir.path().join("ws");
        std::fs::create_dir_all(ws.join(".git/hooks")).unwrap();
        std::fs::write(ws.join(".git/config"), "x").unwrap();
        let outside_file = dir.path().join("f");

        // inside workspace: allowed (also via a relative path and a subdir)
        assert!(ok(&run(
            &ws,
            "echo a > a.txt && mkdir d && echo b > d/b.txt"
        )
        .await));
        // outside workspace: denied, and nothing was created
        assert!(!ok(&run(
            &ws,
            &format!("echo x > {}", outside_file.display())
        )
        .await));
        assert!(!outside_file.exists());
        // `..` escape
        assert!(!ok(&run(&ws, "echo x > ../escape.txt").await));
        assert!(!dir.path().join("escape.txt").exists());
        // symlink inside ws pointing outside
        std::os::unix::fs::symlink(&outside_file, ws.join("link")).unwrap();
        assert!(!ok(&run(&ws, "echo x > link").await));
        assert!(!outside_file.exists());
        // git hook / config are read-only, rest of .git is writable
        assert!(!ok(&run(&ws, "echo x > .git/hooks/pre-commit").await));
        assert!(!ok(&run(&ws, "echo x >> .git/config").await));
        assert!(ok(&run(&ws, "echo x > .git/HEAD").await));
        // network is off
        let net = run(
            &ws,
            "/usr/bin/curl -sS -m 5 -o /dev/null https://example.com",
        )
        .await;
        assert!(!ok(&net));
        // credential dirs are unreadable (only checked when they exist on this machine)
        let ssh = dirs::home_dir().unwrap().join(".ssh");
        if ssh.exists() {
            assert!(!ok(&run(&ws, &format!("ls {}", ssh.display())).await));
        }
    }

    #[tokio::test]
    async fn preserves_env_cwd_and_exit_code() {
        let dir = tempfile::tempdir().unwrap();
        let mut c = Command::new("/bin/sh");
        c.arg("-c")
            .arg("test \"$PLEUM_T\" = 1 && test -z \"${PLEUM_GONE:-}\" && pwd -P && exit 7")
            .env("PLEUM_T", "1")
            .env_remove("PLEUM_GONE")
            .current_dir(dir.path());
        let o = wrap(c, dir.path()).unwrap().output().await.unwrap();
        assert_eq!(o.status.code(), Some(7));
        let real = std::fs::canonicalize(dir.path()).unwrap();
        assert_eq!(
            String::from_utf8_lossy(&o.stdout).trim(),
            real.to_str().unwrap()
        );
    }
}

#[cfg(all(test, unix))]
mod check_write_tests {
    use super::*;

    #[test]
    fn allows_workspace_blocks_escapes() {
        // Parent under $HOME so `..` from the workspace is outside both workspace and temp.
        let base = tempfile::Builder::new()
            .prefix("pleum-cw-test")
            .tempdir_in(dirs::home_dir().unwrap())
            .unwrap();
        let ws = base.path().join("ws");
        std::fs::create_dir_all(ws.join(".git/hooks")).unwrap();
        std::fs::write(ws.join(".git/config"), "x").unwrap();
        let outside = base.path().join("outside");
        std::fs::create_dir(&outside).unwrap();

        // allowed: existing file, new file, new nested dirs, other .git files
        std::fs::write(ws.join("a.txt"), "").unwrap();
        for p in ["a.txt", "new.txt", "x/y/z.txt", ".git/HEAD"] {
            assert!(check_write(&ws.join(p), &ws).is_ok(), "{p}");
        }
        // denied: absolute outside, `..` (existing and non-existing tails), git hooks/config
        for p in [
            outside.join("f").to_str().unwrap().to_string(),
            ws.join("../f").to_str().unwrap().to_string(),
            ws.join("nope/../../f").to_str().unwrap().to_string(),
            ws.join(".git/hooks/pre-commit")
                .to_str()
                .unwrap()
                .to_string(),
            ws.join(".git/config").to_str().unwrap().to_string(),
            dirs::home_dir()
                .unwrap()
                .join(".zshrc")
                .to_str()
                .unwrap()
                .to_string(),
        ] {
            assert!(check_write(Path::new(&p), &ws).is_err(), "{p}");
        }
        // symlinks: file link, dir link, dangling link, all pointing outside
        std::os::unix::fs::symlink(outside.join("f"), ws.join("dangling")).unwrap();
        std::os::unix::fs::symlink(&outside, ws.join("dirlink")).unwrap();
        for p in ["dangling", "dirlink/f", "dirlink/new/f"] {
            assert!(check_write(&ws.join(p), &ws).is_err(), "{p}");
        }
    }
}
