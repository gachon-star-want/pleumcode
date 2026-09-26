//! OS-level sandbox for commands spawned by agent tools (pleumcode).
//!
//! This is a *security boundary*, unlike `tool_permissions` / approval modes,
//! which are guardrails that `bash -c`, aliases or quoting can bypass. It is
//! deliberately not configurable at runtime: no env var, no config key.
//!
//! Policy (Codex-style split: the agent process itself stays unsandboxed so it
//! can reach the model API; only the commands it spawns are confined):
//! - writes only inside the workspace and the temp dir (`.git/hooks` and
//!   `.git/config` stay read-only: a hook is a later, unsandboxed host exec).
//!   On Linux the temp dir is a private tmpfs, not the host's.
//! - no network at all
//! - credential dirs (`~/.ssh`, `~/.aws`, `~/.gnupg`, pleum config) unreadable,
//!   since anything a command prints ends up in the model's context
//! - environment is not inherited wholesale: only a small non-secret allowlist
//!   (`ENV_ALLOWLIST`) plus whatever the caller explicitly set on `cmd` — the
//!   pleumcode process's own env holds `PLEUM_API_KEY`, and neither backend's
//!   namespace/profile isolation touches envp, so without this a command could
//!   just run `env` and hand the key straight back into the model's context
//!
//! Backends: macOS Seatbelt (`sandbox-exec`) and Linux bubblewrap (`bwrap`, namespaces:
//! read-only root, private net/pid/ipc). Every other OS, and Linux without `bwrap`, fails
//! closed: `wrap` returns an error and the command is not run.
//! ponytail: the Linux backend has no seccomp/Landlock layer on top of the namespaces yet, so a
//! path-based unix socket outside /run and /tmp (e.g. under $HOME) is still reachable.
//! The complete fix is a seccomp filter denying AF_UNIX connect.

use std::io;
use std::path::{Path, PathBuf};
use tokio::process::Command;

/// Env vars inherited from the pleumcode process's own environment into every
/// sandboxed command, on top of whatever the caller explicitly set on `cmd`.
/// Deliberately an allowlist, not a denylist of "secret-shaped" names: a
/// denylist only ever catches patterns someone thought to name (`*_API_KEY`,
/// `*_TOKEN`, ...), while PLEUM_API_KEY and anything else in the process's
/// env stays out unless it's named here.
const ENV_ALLOWLIST: &[&str] = &[
    "PATH", "HOME", "USER", "LOGNAME", "SHELL", "TERM", "TZ", "LANG", "LC_ALL", "LC_CTYPE",
];

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
  (subpath (param "PLEUM_CONFIG")))
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

/// `wrap` rebuilds the sandboxed command from `cmd.as_std().get_args()` — the only
/// public way to read args back off an already-built [`Command`]. That accessor is
/// display-only: on Unix, an arg containing an embedded NUL byte is stored for a
/// real exec-time error but *read back* as the literal text `<string-with-nul>`,
/// silently losing the original bytes rather than surfacing an error. Callers must
/// therefore reject a NUL byte in any string they are about to turn into an `arg()`
/// themselves, before building the `Command` at all — after that point the real
/// bytes are unrecoverable and `wrap` would rebuild the wrong command instead of
/// refusing to run it.
pub fn reject_embedded_nul(s: &str) -> io::Result<()> {
    if s.contains('\0') {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "sandbox: command contains a nul byte",
        ));
    }
    Ok(())
}

/// Returns `cmd` re-targeted so it runs inside the sandbox, confined to `workspace`.
/// `readable` are host paths the command must still be able to read even where the
/// backend hides the host's temp dir (the shell tool's saved full-output files).
/// Program, args, env and cwd are preserved. Fails closed if the sandbox is unavailable.
///
/// Callers must have already run every string they put into `cmd`'s args through
/// [`reject_embedded_nul`] — see its doc comment for why.
pub fn wrap(cmd: Command, workspace: &Path, readable: &[&Path]) -> io::Result<Command> {
    let home = std::env::var_os("HOME")
        .map(PathBuf::from)
        .ok_or_else(|| io::Error::other("sandbox: cannot resolve $HOME"))?;
    wrap_with(
        cmd,
        workspace,
        readable,
        &home,
        &crate::config::paths::Paths::config_dir(),
    )
}

/// `wrap` with `$HOME` and the pleum config dir passed in (both are made unreadable).
fn wrap_with(
    cmd: Command,
    workspace: &Path,
    readable: &[&Path],
    home: &Path,
    config_dir: &Path,
) -> io::Result<Command> {
    #[cfg(any(target_os = "macos", target_os = "linux"))]
    {
        // Both backends match canonical paths (macOS `/private/var`, Linux bind mounts).
        let real = |p: &Path| std::fs::canonicalize(p);
        let ws = real(workspace)?;
        let tmp = real(&std::env::temp_dir())?;
        let home = real(home).unwrap_or_else(|_| home.to_path_buf());
        // May not exist yet.
        let config_dir = real(config_dir).unwrap_or_else(|_| config_dir.to_path_buf());
        let std_cmd = cmd.as_std();

        // macOS reads everything already and denies unix-socket connects by default.
        #[cfg(target_os = "macos")]
        let _ = readable;
        #[cfg(target_os = "macos")]
        let mut out = {
            let mut out = Command::new(SANDBOX_EXEC);
            out.arg("-p").arg(PROFILE);
            for (k, v) in [
                ("WS", &ws),
                ("TMP", &tmp),
                ("HOME", &home),
                ("PLEUM_CONFIG", &config_dir),
            ] {
                out.arg("-D").arg(format!("{k}={}", v.display()));
            }
            if let Some(dir) = std_cmd.get_current_dir() {
                out.current_dir(dir);
            }
            out
        };

        #[cfg(target_os = "linux")]
        let mut out = {
            let bwrap = which::which("bwrap").map_err(|_| {
                io::Error::new(
                    io::ErrorKind::NotFound,
                    "sandbox: `bwrap` (bubblewrap) not found; install it (e.g. `apt install bubblewrap`)",
                )
            })?;
            let mut out = Command::new(bwrap);
            // Read-only view of the whole filesystem, private net/pid/ipc/uts, no capabilities.
            out.args([
                "--unshare-all",
                "--die-with-parent",
                "--new-session",
                "--cap-drop",
                "ALL",
            ])
            .args(["--ro-bind", "/", "/", "--dev", "/dev", "--proc", "/proc"]);
            // `--ro-bind / /` stops writes but not connect() to path-based unix sockets
            // (docker.sock, the systemd/dbus session bus, ssh-agent, X11), so hide the places
            // they live: /run and a private /tmp instead of the host's. Order matters: these
            // tmpfs mounts come first so the workspace and readable paths bind on top.
            out.args(["--tmpfs", "/run"]);
            out.arg("--tmpfs").arg(&tmp);
            out.arg("--bind").arg(&ws).arg(&ws);
            for p in readable {
                let p = real(p)?;
                out.arg("--ro-bind").arg(&p).arg(&p);
            }
            // Later mounts win: re-lock the git exec paths, then blank the credential dirs.
            for rel in [".git/hooks", ".git/config"] {
                let p = ws.join(rel);
                if p.exists() {
                    out.arg("--ro-bind").arg(&p).arg(&p);
                }
            }
            let secrets = [".ssh", ".aws", ".gnupg"].map(|d| home.join(d));
            for p in secrets.iter().chain([&config_dir]) {
                if p.is_dir() {
                    out.arg("--tmpfs").arg(p);
                }
            }
            if let Some(dir) = std_cmd.get_current_dir() {
                out.arg("--chdir").arg(dir);
            }
            out.arg("--");
            out
        };

        out.arg(std_cmd.get_program()).args(std_cmd.get_args());
        // Start from nothing, not "inherit everything" (Command's default): see
        // `ENV_ALLOWLIST` above for why. `PLEUM_CONFIG` above is this sandbox's own
        // `-D` parameter and unrelated to the child's env.
        out.env_clear();
        for name in ENV_ALLOWLIST {
            if let Some(value) = std::env::var_os(name) {
                out.env(name, value);
            }
        }
        // The caller's own explicit `.env()`/`.env_remove()` calls (e.g. PLUGIN_ROOT,
        // AGENT_SESSION_ID, an explicitly resolved PATH) are a deliberate ask and win
        // over the allowlist above.
        for (k, v) in std_cmd.get_envs() {
            match v {
                Some(v) => out.env(k, v),
                None => out.env_remove(k),
            };
        }
        Ok(out)
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        let _ = (cmd, workspace, readable, home, config_dir);
        Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "sandbox: no OS sandbox backend for this platform; refusing to run command unconfined",
        ))
    }
}

/// In-process counterpart of the OS write policy, for tools that write files
/// inside the agent process (where `wrap` can't reach). Same rules: workspace + temp
/// dir only, `.git/hooks` and `.git/config` excluded. Keep in sync with `PROFILE` and the bwrap mounts.
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

#[cfg(all(test, any(target_os = "macos", target_os = "linux")))]
mod tests {
    use super::*;
    use std::process::Output;

    async fn run(ws: &Path, script: &str) -> Output {
        let mut c = Command::new("/bin/sh");
        c.arg("-c").arg(script).current_dir(ws);
        wrap(c, ws, &[]).unwrap().output().await.unwrap()
    }

    fn ok(o: &Output) -> bool {
        o.status.success()
    }

    #[tokio::test]
    async fn confines_writes_network_and_secrets() {
        // `wrap` now reads PATH/HOME/etc for the env allowlist; serialize against
        // any other test (e.g. hooks::tests::command_hooks_repair_path_when_enabled)
        // that mutates those same process-global vars under env_lock.
        let _env_guard = env_lock::lock_env(std::iter::empty::<(&str, Option<&str>)>());
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
        #[cfg(target_os = "macos")]
        assert!(!ok(&run(
            &ws,
            "/usr/bin/curl -sS -m 5 -o /dev/null https://example.com"
        )
        .await));
        // Private net namespace: a different namespace id than ours (a new netns may still
        // list DOWN tunnel devices like tunl0/sit0, so the interface count proves nothing).
        #[cfg(target_os = "linux")]
        {
            let host = std::fs::read_link("/proc/self/ns/net").unwrap();
            let o = run(&ws, "readlink /proc/self/ns/net").await;
            let inside = String::from_utf8_lossy(&o.stdout).trim().to_string();
            assert!(ok(&o) && !inside.is_empty());
            assert_ne!(inside, host.to_string_lossy());
        }
    }

    #[tokio::test]
    async fn credential_dirs_are_unreadable() {
        // `wrap` now reads PATH/HOME/etc for the env allowlist; serialize against
        // any other test (e.g. hooks::tests::command_hooks_repair_path_when_enabled)
        // that mutates those same process-global vars under env_lock.
        let _env_guard = env_lock::lock_env(std::iter::empty::<(&str, Option<&str>)>());
        let dir = tempfile::tempdir().unwrap();
        let (home, ws, cfg) = (
            dir.path().join("home"),
            dir.path().join("ws"),
            dir.path().join("cfg"),
        );
        std::fs::create_dir(&ws).unwrap();
        for d in [
            home.join(".ssh"),
            home.join(".aws"),
            home.join(".gnupg"),
            cfg.clone(),
        ] {
            std::fs::create_dir_all(&d).unwrap();
            std::fs::write(d.join("secret"), "TOPSECRET").unwrap();
        }
        let mut ok_cmd = Command::new("/bin/sh");
        ok_cmd.arg("-c").arg("true").current_dir(&ws);
        let o = wrap_with(ok_cmd, &ws, &[], &home, &cfg)
            .unwrap()
            .output()
            .await
            .unwrap();
        assert!(o.status.success(), "control command must run: {o:?}");
        for d in [
            home.join(".ssh"),
            home.join(".aws"),
            home.join(".gnupg"),
            cfg.clone(),
        ] {
            let mut c = Command::new("/bin/sh");
            c.arg("-c")
                .arg(format!("cat {}/secret", d.display()))
                .current_dir(&ws);
            let o = wrap_with(c, &ws, &[], &home, &cfg)
                .unwrap()
                .output()
                .await
                .unwrap();
            assert!(!o.status.success(), "{} readable", d.display());
            assert!(!String::from_utf8_lossy(&o.stdout).contains("TOPSECRET"));
        }
    }

    // The host's temp dir (where unix sockets like ssh-agent/X11 often live) is not visible;
    // a `readable` path under it is, read-only.
    #[cfg(target_os = "linux")]
    #[tokio::test]
    async fn linux_temp_is_private_and_readable_paths_are_read_only() {
        // `wrap` now reads PATH/HOME/etc for the env allowlist; serialize against
        // any other test (e.g. hooks::tests::command_hooks_repair_path_when_enabled)
        // that mutates those same process-global vars under env_lock.
        let _env_guard = env_lock::lock_env(std::iter::empty::<(&str, Option<&str>)>());
        let base = tempfile::Builder::new()
            .prefix("pleum-sbx-ws")
            .tempdir_in(dirs::home_dir().unwrap())
            .unwrap();
        let ws = base.path().join("ws");
        std::fs::create_dir(&ws).unwrap();
        let host_only = tempfile::NamedTempFile::new().unwrap();
        let out_dir = tempfile::tempdir().unwrap();
        std::fs::write(out_dir.path().join("full-output"), "FULL").unwrap();

        let run = |script: String| {
            let mut c = Command::new("/bin/sh");
            c.arg("-c").arg(script).current_dir(&ws);
            let mut c = wrap(c, &ws, &[out_dir.path()]).unwrap();
            async move { c.output().await.unwrap() }
        };
        assert!(ok(&run("true".into()).await), "control command must run");
        // host temp file is invisible
        assert!(!ok(
            &run(format!("test -e {}", host_only.path().display())).await
        ));
        // readable path works, is read-only
        let cat = run(format!("cat {}/full-output", out_dir.path().display())).await;
        assert_eq!(String::from_utf8_lossy(&cat.stdout), "FULL");
        assert!(!ok(&run(format!(
            "echo x > {}/new",
            out_dir.path().display()
        ))
        .await));
        // sandbox scratch space in /tmp still works
        assert!(ok(&run("echo x > \"$(mktemp)\"".into()).await));
    }

    #[tokio::test]
    async fn preserves_env_cwd_and_exit_code() {
        // `wrap` now reads PATH/HOME/etc for the env allowlist; serialize against
        // any other test (e.g. hooks::tests::command_hooks_repair_path_when_enabled)
        // that mutates those same process-global vars under env_lock.
        let _env_guard = env_lock::lock_env(std::iter::empty::<(&str, Option<&str>)>());
        let dir = tempfile::tempdir().unwrap();
        let mut c = Command::new("/bin/sh");
        c.arg("-c")
            .arg("test \"$PLEUM_T\" = 1 && test -z \"${PLEUM_GONE:-}\" && pwd -P && exit 7")
            .env("PLEUM_T", "1")
            .env_remove("PLEUM_GONE")
            .current_dir(dir.path());
        let o = wrap(c, dir.path(), &[]).unwrap().output().await.unwrap();
        assert_eq!(o.status.code(), Some(7));
        let real = std::fs::canonicalize(dir.path()).unwrap();
        assert_eq!(
            String::from_utf8_lossy(&o.stdout).trim(),
            real.to_str().unwrap()
        );
    }

    /// A sandboxed command must not be able to read `PLEUM_API_KEY` (or any other
    /// var outside `ENV_ALLOWLIST`) back out via `env` — that's the whole point of
    /// `env_clear` in `wrap_with`: the process's own env is not the child's env.
    #[tokio::test]
    async fn secrets_outside_the_allowlist_are_not_inherited() {
        let _env_guard = env_lock::lock_env([
            ("PLEUM_API_KEY", Some("s3cr3t-token")),
            ("PATH", std::env::var("PATH").ok().as_deref()),
        ]);
        let dir = tempfile::tempdir().unwrap();
        let mut c = Command::new("/bin/sh");
        c.arg("-c").arg("env").current_dir(dir.path());
        let o = wrap(c, dir.path(), &[]).unwrap().output().await.unwrap();
        let stdout = String::from_utf8_lossy(&o.stdout);
        assert!(!stdout.contains("PLEUM_API_KEY"), "leaked env: {stdout}");
        assert!(!stdout.contains("s3cr3t-token"), "leaked env: {stdout}");
        // A sanity check that the child ran at all and PATH (allowlisted) did come through.
        assert!(stdout.contains("PATH="), "got: {stdout}");
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
