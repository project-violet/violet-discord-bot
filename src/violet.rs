use std::{
    env,
    time::{SystemTime, UNIX_EPOCH},
};

use chrono::DateTime;
use itertools::Itertools;
use serde_json::Value;
use sha2::{Digest, Sha512};

fn valid_domain(path: &str) -> String {
    if env::var("ON_SERVER").is_ok() {
        format!("http://127.0.0.1:7788{path}")
    } else {
        format!("https://koromo.xyz/api{path}")
    }
}

fn create_hmac(salt: &str) -> (String, String) {
    let mut hasher = Sha512::new();
    let start = SystemTime::now();
    let since_the_epoch = start
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards");

    let timestamp = since_the_epoch.as_millis();
    hasher.update(format!("{timestamp:?}{salt}"));
    let hash = hasher.finalize();

    let vtoken = timestamp.to_string();
    let vvalid = (format!("{:x}", hash)[..7]).to_string();

    (vtoken, vvalid)
}

fn get1() -> (String, String) {
    let salt = env::var("SALT").expect("token");

    create_hmac(&salt[..])
}

fn get2() -> (String, String) {
    let salt = env::var("WSALT").expect("token");

    create_hmac(&salt[..])
}

pub async fn request_rank() -> eyre::Result<String> {
    let response = reqwest::get(valid_domain("/top?offset=0&count=10&type=daily")).await?;

    if !response.status().is_success() {
        eyre::bail!("Request rank error!");
    }

    let body = response.text().await?;
    let result: Value = serde_json::from_str(&body[..])?;
    let result = result["result"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| {
            let id = e.as_array().unwrap()[0].as_i64().unwrap();
            let cnt = e.as_array().unwrap()[1].as_i64().unwrap();

            format!("{id}({cnt})")
        })
        .enumerate()
        .map(|(index, e)| format!("{}. {e}", index + 1))
        .join("\n");

    Ok(result)
}

pub async fn request_comments() -> eyre::Result<String> {
    let response = reqwest::get(valid_domain(
        "/community/anon/artistcomment/read?name=global_general",
    ))
    .await?;

    if !response.status().is_success() {
        eyre::bail!("Comment request error!");
    }

    let body = response.text().await?;
    let result: Value = serde_json::from_str(&body[..])?;
    let result = result["result"]
        .as_array()
        .unwrap()
        .iter()
        .take(10)
        .rev()
        .map(|e| {
            let author = &e["UserAppId"].as_str().unwrap()[..8];
            let timestamp = DateTime::parse_from_rfc3339(e["TimeStamp"].as_str().unwrap())
                .unwrap()
                .format("%Y-%m-%d %H:%M:%S")
                .to_string();
            let body = e["Body"].as_str().unwrap();

            format!("{author} ({timestamp})\n{body}\n\n")
        })
        .join("");

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::request_comments;

    #[tokio::test]
    async fn unittest_request_comments() -> eyre::Result<()> {
        request_comments().await?;
        Ok(())
    }
}
