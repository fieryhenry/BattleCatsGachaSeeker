use hmac::{Hmac, Mac};
use rand::Rng;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use serde_json::json;
use sha2::Sha256;
use std::time::SystemTime;

async fn get_inquiry_code() -> String {
    let url: &str = "https://nyanko-backups.ponosgames.com/?action=createAccount&referenceId=";
    let res: reqwest::Response = reqwest::get(url).await.unwrap();
    let body: String = res.text().await.unwrap();
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    let inquiry_code: String = json["accountId"].to_string().replace("\"", "");
    inquiry_code
}

fn get_timestamp() -> i32 {
    let start: SystemTime = SystemTime::now();
    let since_the_epoch = start
        .duration_since(SystemTime::UNIX_EPOCH)
        .expect("Time went backwards");
    since_the_epoch.as_secs() as i32
}

fn get_random_hex_str(length: i32) -> String {
    let mut bytes = vec![0u8; length as usize];
    rand::thread_rng().fill(&mut bytes[..]);
    let hex_str: String = hex::encode(bytes);
    hex_str
}

fn generate_signature(inquiry_code: String, data: String) -> String {
    let random_data: String = get_random_hex_str(32);
    let key: String = format!("{}{}", inquiry_code, random_data);
    type HmacSha256 = Hmac<Sha256>;
    let mut hmac = HmacSha256::new_from_slice(key.as_bytes()).unwrap();
    hmac.update(data.as_bytes());
    let result = hmac.finalize();
    let signature = result.into_bytes();
    let signature: String = format!("{:x}", signature);
    format!("{}{}", random_data, signature)
}

fn get_headers(inquiry_code: String, data: String) -> HeaderMap {
    let mut headers = HeaderMap::new();
    let signature: String = generate_signature(inquiry_code, data);
    headers.insert(
        HeaderName::from_static("nyanko-signature"),
        HeaderValue::from_str(signature.as_str()).unwrap(),
    );
    headers.insert(
        HeaderName::from_static("nyanko-timestamp"),
        HeaderValue::from_str(get_timestamp().to_string().as_str()).unwrap(),
    );
    headers.insert(
        HeaderName::from_static("nyanko-signature-version"),
        HeaderValue::from_str("1").unwrap(),
    );
    headers.insert(
        HeaderName::from_static("nyanko-signature-algorithm"),
        HeaderValue::from_str("HMACSHA256").unwrap(),
    );
    headers.insert(
        HeaderName::from_static("accept-enconding"),
        HeaderValue::from_str("gzip").unwrap(),
    );
    headers.insert(
        HeaderName::from_static("connection"),
        HeaderValue::from_str("keep-alive").unwrap(),
    );
    headers.insert(
        HeaderName::from_static("user-agent"),
        HeaderValue::from_str("Dalvik/2.1.0 (Linux; U; Android 9; Pixel 2 Build/PQ3A.190801.002)")
            .unwrap(),
    );
    headers.insert(
        HeaderName::from_static("content-type"),
        HeaderValue::from_str("application/json").unwrap(),
    );
    headers
}

async fn get_password(inquiry_code: String) -> String {
    let url: &str = "https://nyanko-auth.ponosgames.com/v1/users";
    let json = json!({
        "accountCode": inquiry_code,
        "accountCreatedAt": get_timestamp().to_string(),
        "nonce": get_random_hex_str(16),
    });
    let client: reqwest::Client = reqwest::Client::new();
    let headers = get_headers(inquiry_code, json.to_string());
    let res = client
        .post(url)
        .body(json.to_string())
        .headers(headers)
        .send()
        .await
        .unwrap();
    let body = res.text().await.unwrap();
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    let payload: serde_json::Value = json["payload"].to_owned();
    let password: String = payload["password"].to_string().replace("\"", "");
    password
}

fn get_client_info(cc: &str) -> serde_json::Value {
    let data: serde_json::Value = json!({
        "clientInfo": {
            "client": {
                "countryCode": cc.replace("jp", "ja"),
                "version": "120200",
            },
            "device": {
                "model": "SM-G955F"
            },
            "os": {
                "type": "android",
                "version": "9"
            }
        },
        "nonce": get_random_hex_str(16),
    });
    data
}

async fn get_token(cc: &str) -> String {
    let inquiry_code: String = get_inquiry_code().await;
    let password: String = get_password(inquiry_code.clone()).await;
    let mut client_info: serde_json::Value = get_client_info(cc);

    let url: &str = "https://nyanko-auth.ponosgames.com/v1/tokens";
    client_info["password"] = serde_json::Value::String(password);
    client_info["accountCode"] = serde_json::Value::String(inquiry_code.clone());
    let headers = get_headers(inquiry_code, client_info.to_string());

    let client = reqwest::Client::new();
    let res = client
        .post(url)
        .body(client_info.to_string())
        .headers(headers)
        .send()
        .await
        .unwrap();
    let body = res.text().await.unwrap();
    let json: serde_json::Value = serde_json::from_str(&body).unwrap();
    let payload: serde_json::Value = json["payload"].to_owned();
    let token: String = payload["token"].to_string().replace("\"", "");
    token
}

pub async fn get_event_data(cc: &str) -> String {
    let token: String = get_token(cc).await;
    let cc_code: String = cc.replace("jp", "");
    let base_url: String = format!(
        "https://nyanko-events.ponosgames.com/battlecats{}_production/gatya.tsv",
        cc_code,
    );
    let url: String = format!("{}?jwt={}", base_url, token);
    let client: reqwest::Client = reqwest::Client::new();
    let res: reqwest::Response = client.get(url).send().await.unwrap();
    let body: String = res.text().await.unwrap();
    body
}
