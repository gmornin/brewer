const UNITS: &[&str] = &["kB", "MB", "GB", "TB"];

pub fn filesize(bytes: u64) -> String {
    if bytes < 1024 {
        return format!("{:.0} B", bytes);
    }

    let mut bytes: f64 = bytes as f64 / 1024.;
    for unit in UNITS.iter() {
        if bytes < 1024. {
            return format!("{:.2} {unit}", bytes);
        }

        bytes /= 1024.
    }

    format!("{:.2} PB", bytes)
}
