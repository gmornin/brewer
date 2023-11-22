use std::{env, error::Error, fmt::Display, path::Path};

use log::*;
use reqwest::{
    header::{self, HeaderValue, COOKIE, USER_AGENT},
    multipart::{Form, Part},
    StatusCode,
};
use serde::{de::DeserializeOwned, Serialize};
use tokio::{
    fs,
    io::{AsyncReadExt, AsyncWriteExt},
};

use crate::{
    exit_codes::{bad_url, download_failed, file_not_found, sync_failed},
    functions::get_instance,
    CREDS, DOWNLOAD_RETRIES, EXPECT, HTTP,
};

const INSECURE_WARN: &str = "This request is sent using the insecure http protocol";
const SENDING: &str = "Sending request";

pub async fn post<R: DeserializeOwned, T: Serialize + Sized>(
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
    let res = reqwest::Client::new()
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
        .send()
        .await;
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
    let text = res.text().await.unwrap();
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

pub async fn get_string(
    raw_url: &str,
    html: bool,
    token: bool,
) -> Result<(String, StatusCode), RequestError> {
    if raw_url.starts_with("http://") {
        debug!("{INSECURE_WARN}");
    }

    debug!("{SENDING} with GET to {raw_url}");
    let mut builder = reqwest::Client::new().get(raw_url).header(
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

    let res = builder.send().await;
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
    let text = res.text().await.unwrap();
    trace!("Revieved response:\n{text}");
    Ok((text, status))
}

pub async fn get<R: DeserializeOwned>(url: &str) -> Result<R, RequestError> {
    let url = format!(
        "{}://{url}",
        if *HTTP.get().unwrap() {
            "http"
        } else {
            "https"
        }
    );
    let text = get_string(&url, false, false).await?.0;
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

pub async fn upload<R: DeserializeOwned>(url: &str, path: &Path) -> Result<R, RequestError> {
    if !tokio::fs::try_exists(path).await.unwrap() {
        error!("No file at {}", path.to_string_lossy());
        file_not_found(path)
    }

    let url = format!(
        "{}://{url}",
        if *HTTP.get().unwrap() {
            "http"
        } else {
            "https"
        }
    );

    trace!("Starting upload for {} to {url}.", path.to_string_lossy());

    trace!("Reading file content.");
    let mut file = match tokio::fs::OpenOptions::new().read(true).open(path).await {
        Ok(file) => file,
        Err(e) => {
            sync_failed(e.into());
            unreachable!()
        }
    };

    let mut bytes = Vec::new();
    if let Err(e) = file.read_to_end(&mut bytes).await {
        sync_failed(e.into());
        unreachable!()
    }

    let form = Form::new().part(
        "file",
        Part::bytes(bytes)
            .file_name("filename.txt")
            .mime_str("application/octet-stream")
            .unwrap(),
    );

    trace!("Starting upload");

    let res = reqwest::Client::new()
        .post(&url)
        .multipart(form)
        .header(
            USER_AGENT,
            &format!(
                "{} {} (git {})",
                env!("CARGO_PKG_NAME"),
                env!("CARGO_PKG_VERSION"),
                env!("GIT_HASH")
            ),
        )
        .send()
        .await
        .map_err(|e| RequestError::Send {
            url: url.to_string(),
            error: e,
        })?;

    debug!(
        "Recieved response status code {}.",
        res.status().to_string()
    );

    let text = res.text().await.map_err(|e| RequestError::Send {
        url: url.to_string(),
        error: e,
    })?;

    trace!("Deserializing response.");

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

pub async fn get_url(path: &str) -> String {
    format!("{}{path}", get_instance().await)
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

pub async fn download_raw(raw_url: &str, path: &Path) -> Result<(), Box<dyn Error>> {
    trace!(
        "Downloading file from {raw_url} to {}.",
        path.to_string_lossy()
    );
    let mut file = tokio::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .create(true)
        .open(path)
        .await?;

    let mut err = None;
    for retry in 0..*DOWNLOAD_RETRIES.get().unwrap() {
        trace!("Downloading, retry {}.", retry + 1);
        let res = reqwest::get(raw_url).await;
        let res = match res {
            Ok(res) => res,
            Err(e) => {
                debug!("Downloading failed at retry {}: {e}.", retry + 1);
                err = Some(e);
                continue;
            }
        };

        let status = res.status();
        trace!("Request returned statuse code {}", status.to_string());

        let bytes = res.bytes().await?;
        if !status.is_success() {
            fs::remove_file(path).await?;
            download_failed(&path.to_string_lossy(), &String::from_utf8(bytes.to_vec())?);
        }

        trace!("Saving to file.");
        file.write_all(&bytes).await?;
        return Ok(());
    }

    Err(err.unwrap().into())
}

pub async fn download(url: &str, path: &Path) -> Result<(), Box<dyn Error>> {
    let url = format!(
        "{}://{url}",
        if *HTTP.get().unwrap() {
            debug!("{INSECURE_WARN}");
            "http"
        } else {
            "https"
        }
    );

    // println!("{} {}", url, path.to_string_lossy());
    download_raw(&url, path).await
}
