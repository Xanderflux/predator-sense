use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

/// Index of `/sys/class/hwmon` entries grouped by driver name. Discovered
/// lazily once per process — hwmon numbering is stable for a kernel boot.
pub fn index() -> &'static HashMap<String, Vec<PathBuf>> {
    static IDX: OnceLock<HashMap<String, Vec<PathBuf>>> = OnceLock::new();
    IDX.get_or_init(|| {
        let mut map: HashMap<String, Vec<PathBuf>> = HashMap::new();
        if let Ok(rd) = fs::read_dir("/sys/class/hwmon") {
            let mut entries: Vec<_> = rd.flatten().map(|e| e.path()).collect();
            entries.sort();
            for p in entries {
                if let Ok(n) = fs::read_to_string(p.join("name")) {
                    map.entry(n.trim().to_string()).or_default().push(p);
                }
            }
        }
        map
    })
}

pub fn read_temp_milli(path: &Path) -> Option<f64> {
    Some(fs::read_to_string(path).ok()?.trim().parse::<f64>().ok()? / 1000.0)
}

pub fn label_temp(driver: &str, label: &str) -> Option<f64> {
    for p in index().get(driver)? {
        for i in 1..=20 {
            if let Ok(l) = fs::read_to_string(p.join(format!("temp{}_label", i))) {
                if l.trim() == label {
                    return read_temp_milli(&p.join(format!("temp{}_input", i)));
                }
            }
        }
    }
    None
}

pub fn first_temp(driver: &str, file: &str) -> Option<f64> {
    index().get(driver).and_then(|v| v.first()).and_then(|p| read_temp_milli(&p.join(file)))
}

pub fn nth_temp(driver: &str, file: &str, n: usize) -> Option<f64> {
    index().get(driver).and_then(|v| v.get(n)).and_then(|p| read_temp_milli(&p.join(file)))
}
