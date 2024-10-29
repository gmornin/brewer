use std::any::Any;
use std::path::PathBuf;

use chrono::{Datelike, Local, TimeZone, Timelike};
use goodmorning_bindings::services::v1::{ItemVisibility, V1DirItem, V1Job, V1TexUserPublish};
use goodmorning_bindings::structs::TexCompileDisplay;
use goodmorning_bindings::traits::SerdeAny;

use crate::BASE_PATH;
use crate::FULLPATH;
use crate::{functions::*, HTTP};

pub fn diritems_tostring(items: &[V1DirItem]) -> String {
    let longest_size = if items.is_empty() {
        0
    } else {
        // items.sort_by(|this, other| this.name.cmp(&other.name));
        items
            .iter()
            .max_by(|this, other| this.size.cmp(&other.size))
            .unwrap()
            .size
            .to_string()
            .len()
    };

    let base_path = BASE_PATH.get().unwrap();
    let title = format!("{BLUE}{} items in {RESET_COLOUR}{base_path}", items.len());
    let items = items
        .iter()
        .map(|item| {
            diritem_tostring(
                item,
                longest_size,
                if unsafe { *FULLPATH.get().unwrap() } {
                    PathBuf::from(base_path)
                } else {
                    PathBuf::new()
                },
            )
        })
        .collect::<Vec<_>>();
    format!(
        "{title}\n{GREY}{}{RESET_COLOUR}\n{}",
        "─".repeat(items.first().unwrap_or(&title).len()),
        items.join("\n")
    )
}

pub fn diritem_tostring(item: &V1DirItem, max_size_len: usize, path: PathBuf) -> String {
    let visibility = format!(
        "{}{: <8}",
        if item.visibility.inherited {
            GREY
        } else {
            CYAN
        },
        match item.visibility.visibility {
            ItemVisibility::Hidden => "hidden",
            ItemVisibility::Public => "public",
            ItemVisibility::Private => "private",
        }
    );
    let size = item.size.to_string();
    let size_pad = " ".repeat(max_size_len - size.len());

    let localtime = Local.timestamp_opt(item.last_modified as i64, 0).unwrap();
    let min = format!("{:0>2}", localtime.minute());
    let hour = format!("{:0>2}", localtime.hour());
    let day = format!("{: <2}", localtime.day());
    let month = format!("{: <4}", month_abbrev(localtime.month() as u8));
    let year = localtime
        .year()
        .to_string()
        .as_bytes()
        .iter()
        .rev()
        .take(2)
        .rev()
        .map(|b| *b as char)
        .collect::<String>();

    format!(
        "{visibility} {PURPLE}{size_pad}{size} {BLUE}{year} {month} {day} {hour}:{min} {}{RESET_COLOUR}",
        if item.is_file { format!("{YELLOW}{}", path.join(&item.name).to_string_lossy()) } else {  format!("{BLUE}{}/", path.join(&item.name).to_string_lossy())}
    )
}

fn month_abbrev(month: u8) -> &'static str {
    match month {
        1 => "Jan",
        2 => "Feb",
        3 => "Mar",
        4 => "Apr",
        5 => "May",
        6 => "Jun",
        7 => "Jul",
        8 => "Aug",
        9 => "Sept",
        10 => "Oct",
        11 => "Nov",
        12 => "Dec",
        _ => unreachable!(),
    }
}

pub fn jobs_to_string(group: &str, jobs: Vec<V1Job>) -> String {
    if jobs.is_empty() {
        return format!("Nothing in {group}");
    }

    let mut out = format!("{group}:\n");
    let len = jobs.len() - 1;
    for (i, job) in jobs.into_iter().enumerate() {
        if i == len {
            out += "  └──";
            let string = job_to_string(job);
            let mut lines = string.lines();

            out += lines.next().unwrap();
            out += "\n";
            lines.for_each(|line| out += &format!("   {line}\n"));
            continue;
        }

        out += "  ├──";
        let string = job_to_string(job);
        let mut lines = string.lines();

        out += lines.next().unwrap();
        out += "\n";
        lines.for_each(|line| out += &format!("  │  {line}\n"));
        continue;
    }

    out
}

fn job_to_string(job: V1Job) -> String {
    format!("[{}] {}", job.id, task_to_string(job.task))
}

fn task_to_string(task: Box<dyn SerdeAny>) -> String {
    let task_any: Box<dyn Any> = task;

    match () {
        _ if let Some(res) = task_any.downcast_ref::<TexCompileDisplay>() => format!(
            "Compiling `{}` with compiler `{:?}`.\n      {:?} -> {:?}",
            res.path, res.compiler, res.from, res.to
        ),
        _ => "Task cannot be displayed".to_string(),
    }
}

pub fn publishes_to_string(publishes: &[V1TexUserPublish], instance: &str, userid: i64) -> String {
    publishes
        .iter()
        .map(|item| {
            publish_to_string(
                item,
                &get_url_instance(
                    &format!("/api/publish/v1/published-file/id/{userid}/{}", item.id),
                    instance,
                ),
            )
        })
        .collect::<Vec<_>>()
        .join(&format!("\n{GREY}──────────────────{RESET_COLOUR}\n"))
}

pub fn publish_to_string(
    V1TexUserPublish {
        id,
        published,
        title,
        desc,
        ext,
    }: &V1TexUserPublish,
    url: &str,
) -> String {
    let localtime = Local.timestamp_opt(*published as i64, 0).unwrap();
    let min = format!("{:0>2}", localtime.minute());
    let hour = format!("{:0>2}", localtime.hour());
    let day = format!("{: <2}", localtime.day());
    let month = format!("{: <4}", month_abbrev(localtime.month() as u8));
    let year = localtime
        .year()
        .to_string()
        .as_bytes()
        .iter()
        .rev()
        .take(2)
        .rev()
        .map(|b| *b as char)
        .collect::<String>();

    let pad = " ".repeat(id.to_string().len() + 3);
    format!("[{id}] {title}\n{pad}Description: {desc}\n{pad}Published: {year} {month} {day} {hour}:{min}\n{pad}Format: {ext}\n{pad}Url: {}://{url}", if *HTTP.get().unwrap() {"http"} else {"https"})
}
