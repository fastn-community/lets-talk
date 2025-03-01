#![deny(unused_crate_dependencies)]
extern crate self as lets_talk;

mod auth;
mod dyte;

mod create_meeting;
mod session;
mod token;

const TALK_TOKEN_COOKIE: &str = "talk-token";

fn create_session_cookie(
    token: &str,
    meeting_id: &str,
    host: &ft_sdk::Host,
    secure: bool,
) -> Result<http::HeaderValue, ft_sdk::Error> {
    let val = format!("{}:{}", meeting_id, token);
    let cookie = cookie::Cookie::build((TALK_TOKEN_COOKIE, val))
        .domain(host.without_port())
        // TODO: make this so that only mountpoint can access the cookie
        .path("/")
        .max_age(cookie::time::Duration::seconds(34560000))
        .same_site(cookie::SameSite::Strict)
        .secure(secure)
        .http_only(secure)
        .build();

    Ok(http::HeaderValue::from_str(cookie.to_string().as_str())?)
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub(crate) struct Config {
    pub allowed_emails: String,
    pub preset_host: String,
    pub preset_participant: String,
    pub require_verification: bool,
    pub secure_sessions: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            // https://dev.dyte.io/presets?orgId=f233a0a4-afa0-4afd-9969-971d77e799d8
            // Ask FifthTry to create custom presets for you
            preset_host: "group_call_host".to_string(),
            preset_participant: "group_call_participant".to_string(),
            require_verification: false,
            secure_sessions: false,
            allowed_emails: "".to_string(),
        }
    }
}

impl ft_sdk::FromRequest for Config {
    fn from_request(req: &http::Request<serde_json::Value>) -> Result<Self, ft_sdk::Error> {
        let app_url: ft_sdk::AppUrl = ft_sdk::AppUrl::from_request(req)?;
        let url = app_url.join("config")?;

        let req = http::Request::builder()
            .uri(url)
            .body(bytes::Bytes::new())?;

        let res = ft_sdk::http::send(req).unwrap();

        serde_json::from_slice(res.body()).map_err(|e| e.into())
    }
}
