use std::process::Command;
use std::sync::Mutex;
use std::time::{Duration, Instant};

#[derive(Debug, Clone, Default)]
pub struct GpuMetrics {
    pub name: String,
    pub driver: String,
    pub vbios: String,
    pub vram_total_mb: u32,
    pub vram_used_mb: u32,
    pub vram_free_mb: u32,
    pub temp: f64,
    pub clock_core_mhz: u32,
    pub clock_mem_mhz: u32,
    pub clock_max_core: u32,
    pub clock_max_mem: u32,
    pub util_gpu_pct: u32,
    pub util_mem_pct: u32,
    pub power_draw_w: f64,
    pub power_limit_w: f64,
    pub power_max_w: f64,
    pub fan_speed_pct: u32,
    pub pstate: String,
    pub pcie_gen: String,
    pub pcie_width: String,
}

impl GpuMetrics {
    pub fn vram_pct(&self) -> f64 {
        if self.vram_total_mb == 0 {
            0.0
        } else {
            self.vram_used_mb as f64 / self.vram_total_mb as f64 * 100.0
        }
    }
    pub fn power_pct(&self) -> f64 {
        if self.power_max_w <= 0.0 {
            0.0
        } else {
            (self.power_draw_w / self.power_max_w * 100.0).clamp(0.0, 100.0)
        }
    }
    pub fn is_present(&self) -> bool {
        !self.name.is_empty() && self.vram_total_mb > 0
    }
}

// nvidia-smi is a fork+exec (~50-100ms). Cache result so multiple consumers
// (gpu_page, usage_page sampler) share a single invocation per refresh window.
static CACHE: Mutex<Option<(Instant, GpuMetrics)>> = Mutex::new(None);
const TTL: Duration = Duration::from_millis(1800);

pub fn read_gpu_metrics() -> GpuMetrics {
    if let Some((t, m)) = CACHE.lock().unwrap().as_ref() {
        if t.elapsed() < TTL {
            return m.clone();
        }
    }
    let m = fetch_gpu_metrics();
    *CACHE.lock().unwrap() = Some((Instant::now(), m.clone()));
    m
}

fn fetch_gpu_metrics() -> GpuMetrics {
    let o = Command::new("nvidia-smi")
        .args([
            "--query-gpu=name,driver_version,vbios_version,memory.total,memory.used,memory.free,temperature.gpu,clocks.gr,clocks.mem,clocks.max.gr,clocks.max.mem,utilization.gpu,utilization.memory,power.draw,power.limit,power.max_limit,fan.speed,pstate,pcie.link.gen.current,pcie.link.width.current",
            "--format=csv,noheader,nounits",
        ])
        .output();
    let o = match o {
        Ok(o) if o.status.success() => o,
        _ => return GpuMetrics::default(),
    };
    let t = String::from_utf8_lossy(&o.stdout);
    let p: Vec<&str> = t.trim().split(", ").collect();
    if p.len() < 20 {
        return GpuMetrics::default();
    }
    let parse_u32 = |s: &str| {
        s.trim()
            .replace(" MiB", "")
            .replace(" MHz", "")
            .replace(" W", "")
            .replace(" %", "")
            .replace("[N/A]", "0")
            .parse::<u32>()
            .unwrap_or(0)
    };
    let parse_f64 = |s: &str| {
        s.trim()
            .replace(" W", "")
            .replace(" C", "")
            .replace("[N/A]", "0")
            .parse::<f64>()
            .unwrap_or(0.0)
    };

    GpuMetrics {
        name: p[0].trim().into(),
        driver: p[1].trim().into(),
        vbios: p[2].trim().into(),
        vram_total_mb: parse_u32(p[3]),
        vram_used_mb: parse_u32(p[4]),
        vram_free_mb: parse_u32(p[5]),
        temp: parse_f64(p[6]),
        clock_core_mhz: parse_u32(p[7]),
        clock_mem_mhz: parse_u32(p[8]),
        clock_max_core: parse_u32(p[9]),
        clock_max_mem: parse_u32(p[10]),
        util_gpu_pct: parse_u32(p[11]),
        util_mem_pct: parse_u32(p[12]),
        power_draw_w: parse_f64(p[13]),
        power_limit_w: parse_f64(p[14]),
        power_max_w: parse_f64(p[15]),
        fan_speed_pct: parse_u32(p[16]),
        pstate: p[17].trim().into(),
        pcie_gen: p[18].trim().into(),
        pcie_width: p[19].trim().into(),
    }
}
