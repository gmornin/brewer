const UNITS: &[(&str, u64)] = &[
    ("day", 86400),
    ("hour", 3600),
    ("minute", 60),
    ("second", 1),
];

pub fn duration_as_string(mut secs: u64) -> String {
    let mut out = Vec::new();

    UNITS.iter().for_each(|(unit, factor)| {
        let quantity = secs / factor;
        if quantity == 0 {
            return;
        }

        if quantity == 1 {
            out.push(format!("{quantity} {unit}"))
        } else {
            out.push(format!("{quantity} {unit}s"))
        }
        secs -= quantity * factor;
    });

    if out.is_empty() {
        "0 seconds".to_string()
    } else if out.len() == 1 {
        out[0].clone()
    } else {
        let last = out.pop().unwrap();
        *out.last_mut().unwrap() = format!("{} and {}", out.last().unwrap(), last);
        out.join(", ")
    }
}
