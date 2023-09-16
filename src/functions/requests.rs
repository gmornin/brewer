use std::{env, error::Error, fmt::Display};

use log::*;
use reqwest::header::USER_AGENT;
use serde::{de::DeserializeOwned, Serialize};

use crate::{functions::instance_or_exit, HTTP};

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
    debug!("Response recieved, deserializing");
    let text = res.text().unwrap_or_default();
    match serde_json::from_str(&text) {
        Ok(out) => Ok(out),
        Err(e) => Err(RequestError::Deserialize {
            url,
            error: e,
            content: text,
        }),
    }
}

pub fn get<R: DeserializeOwned>(url: &str) -> Result<R, RequestError> {
    let url = format!(
        "{}://{url}",
        if *HTTP.get().unwrap() {
            debug!("{INSECURE_WARN}");
            "http"
        } else {
            "https"
        }
    );
    debug!("{SENDING} with GET");
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
        .send();
    let res = match res {
        Ok(res) => res,
        Err(e) => {
            error!("Error sending request to `{url}`");
            return Err(RequestError::Send {
                url: url.to_string(),
                error: e,
            });
        }
    };
    debug!("Response recieved, deserializing");
    let text = res.text().unwrap_or_default();
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
    format!("{}{path}", instance_or_exit())
}
