use gtk4::prelude::*;
use gtk4::{self as gtk};

use crate::hardware::sysinfo::{self, SystemInfo};

/// Dashboard principal: hero com foto do notebook + especificações técnicas.
pub fn build() -> gtk::ScrolledWindow {
    let scroll = gtk::ScrolledWindow::new();
    scroll.set_policy(gtk::PolicyType::Never, gtk::PolicyType::Automatic);
    scroll.set_hexpand(true);
    scroll.set_vexpand(true);

    let info = sysinfo::read_system_info();

    let page = gtk::Box::new(gtk::Orientation::Vertical, 16);
    page.set_margin_top(18);
    page.set_margin_bottom(18);
    page.set_margin_start(24);
    page.set_margin_end(24);

    // === Hero header: foto + nome/modelo ===
    let hero = gtk::Box::new(gtk::Orientation::Horizontal, 24);
    hero.set_halign(gtk::Align::Fill);
    hero.add_css_class("dashboard-hero");

    if let Some(path) = find_resource("Predator PH315-54.png").or_else(|| find_resource("laptop-thumb.png")) {
        let pic = gtk::Picture::for_filename(path);
        pic.set_size_request(320, 200);
        pic.set_can_shrink(true);
        pic.set_valign(gtk::Align::Center);
        hero.append(&pic);
    }

    let hero_info = gtk::Box::new(gtk::Orientation::Vertical, 6);
    hero_info.set_valign(gtk::Align::Center);
    hero_info.set_hexpand(true);

    let vendor = gtk::Label::new(Some(&info.vendor));
    vendor.add_css_class("dashboard-vendor");
    vendor.set_halign(gtk::Align::Start);
    hero_info.append(&vendor);

    let product = gtk::Label::new(Some(&info.product_name));
    product.add_css_class("dashboard-product");
    product.set_halign(gtk::Align::Start);
    product.set_wrap(true);
    hero_info.append(&product);

    let summary = gtk::Label::new(Some(&build_short_summary(&info)));
    summary.add_css_class("dashboard-summary");
    summary.set_halign(gtk::Align::Start);
    summary.set_wrap(true);
    hero_info.append(&summary);

    hero.append(&hero_info);
    page.append(&hero);

    // === Specs grid ===
    let specs_title = gtk::Label::new(Some(crate::i18n::t("dashboard_specs")));
    specs_title.add_css_class("section-title");
    specs_title.set_halign(gtk::Align::Start);
    specs_title.set_margin_top(8);
    page.append(&specs_title);

    let grid = gtk::Grid::new();
    grid.set_column_spacing(12);
    grid.set_row_spacing(12);
    grid.set_column_homogeneous(true);
    grid.set_margin_top(6);

    let cpu_detail = if info.cpu_cores > 0 {
        format!(
            "{}\n{} núcleos / {} threads · {:.2} GHz",
            info.cpu_model,
            info.cpu_cores,
            info.cpu_threads,
            info.cpu_max_freq_mhz as f64 / 1000.0
        )
    } else {
        info.cpu_model.clone()
    };

    let gpu_detail = if info.gpu_vram_mb > 0 {
        format!(
            "{}\n{:.0} GB VRAM · Driver {}",
            info.gpu_name,
            info.gpu_vram_mb as f64 / 1024.0,
            if info.gpu_driver.is_empty() { "—".into() } else { info.gpu_driver.clone() }
        )
    } else {
        info.gpu_name.clone()
    };

    let ram_detail = if info.ram_total_gb > 0.0 {
        if info.ram_type.is_empty() {
            format!("{:.0} GB total", info.ram_total_gb)
        } else {
            format!("{:.0} GB · {}", info.ram_total_gb, info.ram_type)
        }
    } else {
        "—".into()
    };

    let storage_detail = if info.storage.is_empty() {
        "—".into()
    } else {
        info.storage
            .iter()
            .map(|s| format!("{} · {:.0} GB · {}", s.model.trim(), s.size_gb, s.kind))
            .collect::<Vec<_>>()
            .join("\n")
    };

    let net_detail = if info.net_interface.is_empty() {
        "Sem conexão ativa".into()
    } else {
        format!("{} · {}\n{}", info.net_type, info.net_interface, info.net_mac)
    };

    let os_detail = format!("{}\nKernel {}", info.os_pretty, info.kernel);

    let bios_detail = if info.bios_version.is_empty() {
        "—".into()
    } else {
        format!("BIOS {}", info.bios_version)
    };

    let cards = [
        ("CPU", "💻", cpu_detail),
        ("GPU", "🎮", gpu_detail),
        (crate::i18n::t("memory"), "🧠", ram_detail),
        (crate::i18n::t("storage"), "💾", storage_detail),
        (crate::i18n::t("network"), "🌐", net_detail),
        (crate::i18n::t("system_os"), "🐧", os_detail),
        ("BIOS", "⚙", bios_detail),
    ];

    for (i, (title, icon, value)) in cards.iter().enumerate() {
        let card = create_spec_card(icon, title, value);
        let col = (i % 2) as i32;
        let row = (i / 2) as i32;
        grid.attach(&card, col, row, 1, 1);
    }

    page.append(&grid);

    scroll.set_child(Some(&page));
    scroll
}

fn build_short_summary(info: &SystemInfo) -> String {
    let mut parts: Vec<String> = Vec::new();
    if info.cpu_cores > 0 {
        parts.push(format!(
            "{} núcleos / {} threads",
            info.cpu_cores, info.cpu_threads
        ));
    }
    if info.ram_total_gb > 0.0 {
        parts.push(format!("{:.0} GB RAM", info.ram_total_gb));
    }
    if !info.gpu_name.is_empty() && info.gpu_name != "Desconhecida" {
        parts.push(info.gpu_name.clone());
    }
    parts.join(" · ")
}

fn create_spec_card(icon: &str, title: &str, value: &str) -> gtk::Box {
    let card = gtk::Box::new(gtk::Orientation::Horizontal, 12);
    card.add_css_class("spec-card");
    card.set_valign(gtk::Align::Start);

    let icon_l = gtk::Label::new(Some(icon));
    icon_l.add_css_class("spec-icon");
    icon_l.set_valign(gtk::Align::Start);
    card.append(&icon_l);

    let text = gtk::Box::new(gtk::Orientation::Vertical, 2);
    text.set_hexpand(true);

    let t = gtk::Label::new(Some(title));
    t.add_css_class("spec-title");
    t.set_halign(gtk::Align::Start);
    text.append(&t);

    let v = gtk::Label::new(Some(value));
    v.add_css_class("spec-value");
    v.set_halign(gtk::Align::Start);
    v.set_wrap(true);
    v.set_xalign(0.0);
    text.append(&v);

    card.append(&text);
    card
}

fn find_resource(name: &str) -> Option<String> {
    if let Ok(exe) = std::env::current_exe() {
        let dir = exe.parent()?;
        let p = dir.join("../../resources").join(name);
        if p.exists() {
            return Some(p.to_string_lossy().to_string());
        }
        let p = dir.join("resources").join(name);
        if p.exists() {
            return Some(p.to_string_lossy().to_string());
        }
    }
    let dev = format!("/opt/predator-sense/resources/{}", name);
    if std::path::Path::new(&dev).exists() {
        return Some(dev);
    }
    None
}
