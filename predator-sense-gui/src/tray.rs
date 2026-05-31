use std::path::PathBuf;
use std::process::Command;

const LOCK_FILE: &str = "/tmp/predator-sense-tray.lock";
const LOG_FILE: &str = "/tmp/predator-sense-tray.log";

/// Manages the system tray helper process.
/// The tray runs as a detached process so it survives even if the main app hides.
pub struct TrayManager {
    pub started: bool,
}

impl TrayManager {
    pub fn new() -> Self {
        Self { started: false }
    }

    /// Start the tray helper as a detached background process. Idempotente: se já houver
    /// um tray rodando e saudável, não duplica. Se o anterior morreu sem limpar lock, reinicia.
    pub fn start(&mut self) {
        let script = match find_tray_script() {
            Some(s) => s,
            None => {
                eprintln!("[tray] tray_helper.py não encontrado");
                return;
            }
        };

        // Se há um tray rodando com PID válido, nada a fazer.
        if let Some(pid) = live_tray_pid() {
            eprintln!("[tray] já rodando (PID {})", pid);
            self.started = true;
            return;
        }

        // Limpa lock stale
        let _ = std::fs::remove_file(LOCK_FILE);

        // Captura stderr em log pra diagnóstico
        let stderr_stdio = match std::fs::File::create(LOG_FILE) {
            Ok(f) => std::process::Stdio::from(f),
            Err(_) => std::process::Stdio::null(),
        };

        // Usa `setsid --fork` pra criar nova sessão e completamente desacoplar do processo
        // pai (evita o filho virar zombie ao herdar process-group/signals do GTK).
        match Command::new("setsid")
            .arg("--fork")
            .arg("python3")
            .arg("-u") // unbuffered stdout/stderr
            .arg(&script)
            .env("PYTHONUNBUFFERED", "1")
            .stdin(std::process::Stdio::null())
            .stdout(std::process::Stdio::null())
            .stderr(stderr_stdio)
            .spawn()
        {
            Ok(child) => {
                eprintln!("[tray] helper spawned via setsid (launcher pid {})", child.id());
                // Faz reap do launcher imediato pra não virar zombie
                let _ = child.wait_with_output();
                self.started = true;
            }
            Err(e) => eprintln!("[tray] falha ao spawnar: {}", e),
        }
    }
}

/// Lê o PID do lock file e verifica se o processo ainda vive.
fn live_tray_pid() -> Option<u32> {
    let content = std::fs::read_to_string(LOCK_FILE).ok()?;
    let pid: u32 = content.trim().parse().ok()?;
    if pid == 0 {
        return None;
    }
    // /proc/<pid>/comm existe se o processo vive
    if std::path::Path::new(&format!("/proc/{}/comm", pid)).exists() {
        // Confirma que é python (tray_helper.py)
        if let Ok(comm) = std::fs::read_to_string(format!("/proc/{}/comm", pid)) {
            if comm.trim().starts_with("python") {
                return Some(pid);
            }
        }
    }
    None
}

// No Drop implementation - tray process lives independently

fn find_tray_script() -> Option<PathBuf> {
    let candidates = [
        "/opt/predator-sense/tray_helper.py",
        "/opt/predator-sense/resources/tray_helper.py",
    ];

    for path in &candidates {
        let p = PathBuf::from(path);
        if p.exists() {
            return Some(p);
        }
    }

    // Try relative to executable
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            for rel in &["tray_helper.py", "resources/tray_helper.py", "../../resources/tray_helper.py"] {
                let p = dir.join(rel);
                if p.exists() {
                    return p.canonicalize().ok();
                }
            }
        }
    }

    None
}
