use std::sync::atomic::{AtomicBool, Ordering};

static WINDOW_VISIBLE: AtomicBool = AtomicBool::new(true);

pub fn set_window_visible(v: bool) {
    WINDOW_VISIBLE.store(v, Ordering::Relaxed);
}

pub fn is_window_visible() -> bool {
    WINDOW_VISIBLE.load(Ordering::Relaxed)
}
