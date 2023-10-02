use std::{env, error::Error, fmt::Display, fs::OpenOptions, io::Write, path::Path};

use log::*;
use reqwest::{
    header::{self, HeaderValue, COOKIE, USER_AGENT},
    StatusCode,
};
use serde::{de::DeserializeOwned, Serialize};

use crate::{exit_codes::bad_url, functions::get_instance, CREDS, DOWNLOAD_RETRIES, EXPECT, HTTP};

const INSECURE_WARN: &str = "This request is sent using the insecure http protocol";
const SENDING: &str = "Sending request";

pub fn post<R: DeserializeOwned, T: Serialize + Sized>(
    url: &str,
    body: T,
) -> Result<R, RequestError> {
    let url = format!(
        "{}://{url}",
        if *HTTP.get().unwrap() {
            debug!("{INSECURE_WARN}");
            "http"
        } else {
            "https"
        }
    );
    debug!("{SENDING} with POST to {url}");
    debug!("Request body: {}", serde_json::to_string(&body).unwrap());
    let res = reqwest::blocking::Client::new()
        .post(&url)
        .header(
            USER_AGENT,
            &format!(
                "{} {} (git {})",
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_VERSION"),
                env!("GIT_HASH")
            ),
        )
        .json(&body)
        .send();
    let res = match res {
        Ok(res) => res,
        Err(e) => {
            return Err(RequestError::Send {
                url: url.to_string(),
                error: e,
            });
        }
    };

    debug!(
        "recieved response with status code `{}`",
        res.status().to_string()
    );
    debug!("Response recieved, deserializing");
    let text = res.text().unwrap();
    trace!("Revieved response:\n{text}");
    match serde_json::from_str(&text) {
        Ok(out) => Ok(out),
        Err(e) => Err(RequestError::Deserialize {
            url,
            error: e,
            content: text,
        }),
    }
}

pub fn get_string(
    raw_url: &str,
    html: bool,
    token: bool,
) -> Result<(String, StatusCode), RequestError> {
    if raw_url.starts_with("http://") {
        debug!("{INSECURE_WARN}");
    }

    debug!("{SENDING} with GET to {raw_url}");
    let mut builder = reqwest::blocking::Client::new().get(raw_url).header(
        USER_AGENT,
        &format!(
            "{} {} (git {})",
            env!("CARGO_PKG_NAME"),
            env!("CARGO_PKG_VERSION"),
            env!("GIT_HASH")
        ),
    );

    if token {
        builder = builder.header(
            COOKIE,
            HeaderValue::from_str(&format!("token={}", unsafe { &CREDS.get().unwrap().token }))
                .unwrap(),
        );
    }

    if html {
        builder = builder.header(header::ACCEPT, EXPECT);
    }

    let res = builder.send();
    let res = match res {
        Ok(res) => res,
        Err(e) => {
            error!("Error sending request to `{raw_url}`");
            return Err(RequestError::Send {
                url: raw_url.to_string(),
                error: e,
            });
        }
    };
    debug!(
        "recieved response with status code `{}`",
        res.status().to_string()
    );
    debug!("Response recieved, deserializing");

    let status = res.status();
    let text = res.text().unwrap();
    trace!("Revieved response:\n{text}");
    Ok((text, status))
}

pub fn get<R: DeserializeOwned>(url: &str) -> Result<R, RequestError> {
    let url = format!(
        "{}://{url}",
        if *HTTP.get().unwrap() {
            "http"
        } else {
            "https"
        }
    );
    let text = get_string(&url, false, false)?.0;
    match serde_json::from_str(&text) {
        Ok(out) => Ok(out),
        Err(e) => {
            error!("Deserialization failed");
            info!("Server response: \n{}", text);
            Err(RequestError::Deserialize {
                url,
                error: e,
                content: text,
            })
        }
    }
}

#[derive(Debug)]
pub enum RequestError {
    Send {
        url: String,
        error: reqwest::Error,
    },
    Deserialize {
        url: String,
        error: serde_json::Error,
        content: String,
    },
}

impl Display for RequestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Send { url, error } => {
                f.write_fmt(format_args!("error sending request to {url}: {error}"))
            }
            Self::Deserialize {
                url,
                error,
                content,
            } => f.write_fmt(format_args!(
                "error sending request to {url}: {error}\n---\nResponse content:\n{content}"
            )),
        }
    }
}

impl Error for RequestError {}

pub fn get_url(path: &str) -> String {
    format!("{}{path}", get_instance())
}

pub fn get_url_instance(path: &str, instance: &str) -> String {
    format!("{instance}{path}")
}

pub fn url_domain(url: &str) -> &str {
    if let Some(stripped) = url.strip_prefix("http://") {
        debug!("{INSECURE_WARN}");
        stripped
    } else if let Some(stripped) = url.strip_prefix("https://") {
        stripped
    } else {
        bad_url("protocol not specified", url);
        unreachable!()
    }
    .split('/')
    .next()
    .unwrap()
}

pub fn download_raw(raw_url: &str, path: &Path) -> Result<(), Box<dyn Error>> {
    trace!(
        "Downloading file from {raw_url} to {}.",
        path.to_string_lossy()
    );
    let mut file = OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(path)?;

    let mut err = None;
    for retry in 0..*DOWNLOAD_RETRIES.get().unwrap() {
        trace!("Downloading, retry {}.", retry + 1);
        let res = reqwest::blocking::get(raw_url);
        let res = match res {
            Ok(res) => res,
            Err(e) => {
                debug!("Downloading failed at retry {}: {e}.", retry + 1);
                err = Some(e);
                continue;
            }
        };
        trace!("Request returned statuse code {}", res.status().to_string());
        trace!("Saving to file.");
        file.write_all(&res.bytes()?)?;
        return Ok(());
    }

    Err(err.unwrap().into())
}

pub fn download(url: &str, path: &Path) -> Result<(), Box<dyn Error>> {
    let url = format!(
        "{}://{url}",
        if *HTTP.get().unwrap() {
            debug!("{INSECURE_WARN}");
            "http"
        } else {
            "https"
        }
    );

    download_raw(&url, path)
}
