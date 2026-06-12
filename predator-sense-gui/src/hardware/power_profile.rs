//! Automatic performance profile switching based on the power source.
//!
//! When enabled, plugging in AC selects Performance and unplugging selects
//! Balanced. Only acts on transitions so it never fights a manual choice while
//! the source is stable.

use std::fs;
use std::sync::atomic::{AtomicBool, AtomicI8, Ordering};

use super::profile::{set_profile, PowerProfile};

static ENABLED: AtomicBool = AtomicBool::new(false);
/// Last seen AC state: -1 unknown, 0 battery, 1 AC.
static LAST_AC: AtomicI8 = AtomicI8::new(-1);

pub fn set_auto(v: bool) {
    ENABLED.store(v, Ordering::Relaxed);
    // Reset so the next check re-applies for the current source.
    LAST_AC.store(-1, Ordering::Relaxed);
}

pub fn is_auto() -> bool {
    ENABLED.load(Ordering::Relaxed)
}

/// True if AC is connected, reading the first power_supply Mains/ADP device.
pub fn ac_online() -> Option<bool> {
    let rd = fs::read_dir("/sys/class/power_supply").ok()?;
    for e in rd.flatten() {
        let p = e.path();
        let typ = fs::read_to_string(p.join("type")).unwrap_or_default();
        if typ.trim() == "Mains" {
            if let Ok(v) = fs::read_to_string(p.join("online")) {
                return Some(v.trim() == "1");
            }
        }
    }
    None
}

/// Call periodically. Applies the matching profile on a power-source change.
pub fn check() {
    if !is_auto() {
        return;
    }
    let ac = match ac_online() {
        Some(v) => v,
        None => return,
    };
    let cur = if ac { 1 } else { 0 };
    if LAST_AC.swap(cur, Ordering::Relaxed) == cur {
        return; // no transition
    }
    let target = if ac {
        PowerProfile::Performance
    } else {
        PowerProfile::Balanced
    };
    let _ = set_profile(target);
}
