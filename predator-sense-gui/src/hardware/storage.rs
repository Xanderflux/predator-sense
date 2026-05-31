use std::process::Command;

#[derive(Debug, Clone, Default)]
pub struct DiskUsage {
    pub device: String,
    pub fstype: String,
    pub mount: String,
    pub total_bytes: u64,
    pub used_bytes: u64,
    pub avail_bytes: u64,
    pub percent: f64,
}

impl DiskUsage {
    pub fn total_gb(&self) -> f64 {
        self.total_bytes as f64 / 1_073_741_824.0
    }
    pub fn used_gb(&self) -> f64 {
        self.used_bytes as f64 / 1_073_741_824.0
    }
    pub fn avail_gb(&self) -> f64 {
        self.avail_bytes as f64 / 1_073_741_824.0
    }
    pub fn label(&self) -> String {
        if self.mount == "/" {
            "Sistema (/)".into()
        } else if self.mount.starts_with("/boot") {
            format!("Boot · {}", self.mount)
        } else if self.mount.starts_with("/home") {
            format!("Home · {}", self.mount)
        } else if self.mount.starts_with("/mnt") || self.mount.starts_with("/media") {
            format!("Externo · {}", self.mount)
        } else {
            self.mount.clone()
        }
    }
}

/// Lista filesystems relevantes (sem tmpfs/devtmpfs e parecidos).
pub fn list_disks() -> Vec<DiskUsage> {
    let output = Command::new("df")
        .args(["-B1", "--output=source,fstype,size,used,avail,pcent,target"])
        .output();
    let output = match output {
        Ok(o) if o.status.success() => o,
        _ => return Vec::new(),
    };
    let text = String::from_utf8_lossy(&output.stdout);
    let mut disks: Vec<DiskUsage> = Vec::new();
    for (i, line) in text.lines().enumerate() {
        if i == 0 {
            continue;
        }
        let mut it = line.split_whitespace();
        let device = it.next().unwrap_or("").to_string();
        let fstype = it.next().unwrap_or("").to_string();
        let size = it.next().and_then(|v| v.parse().ok()).unwrap_or(0u64);
        let used = it.next().and_then(|v| v.parse().ok()).unwrap_or(0u64);
        let avail = it.next().and_then(|v| v.parse().ok()).unwrap_or(0u64);
        let pcent_raw = it.next().unwrap_or("0%");
        let percent: f64 = pcent_raw.trim_end_matches('%').parse().unwrap_or(0.0);
        let mount: String = it.collect::<Vec<_>>().join(" ");

        // Filtra montagens irrelevantes
        if fstype == "tmpfs"
            || fstype == "devtmpfs"
            || fstype == "squashfs"
            || fstype == "overlay"
            || fstype == "efivarfs"
            || fstype == "autofs"
            || mount.starts_with("/snap")
            || mount.starts_with("/run")
            || mount.starts_with("/proc")
            || mount.starts_with("/sys")
            || mount.starts_with("/dev")
            || size == 0
        {
            continue;
        }
        disks.push(DiskUsage {
            device,
            fstype,
            mount,
            total_bytes: size,
            used_bytes: used,
            avail_bytes: avail,
            percent,
        });
    }
    // Raiz primeiro, depois boot, depois externos
    disks.sort_by_key(|d| match d.mount.as_str() {
        "/" => 0,
        m if m.starts_with("/boot") => 1,
        m if m.starts_with("/home") => 2,
        _ => 3,
    });
    disks
}
