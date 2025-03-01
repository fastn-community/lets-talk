//! This session and dyte session are different. These endpoints are responsible for calling the
//! dyte API to add participants and then handling token in session data (http cookies).
//!
//! The cookies are per meeting, so a new meeting will overwrite the previous cookie.

/// Return the token from active session to join the meeting, or,
/// redirect to /session/new/
#[ft_sdk::processor]
fn session(
    ft_sdk::Cookie(token): ft_sdk::Cookie<{ crate::TALK_TOKEN_COOKIE }>,
    meeting_id: ft_sdk::Query<"meeting-id", String>,
    app_url: ft_sdk::AppUrl,
) -> ft_sdk::processor::Result {
    ft_sdk::println!("======= in session handler ======");
    ft_sdk::println!("token: {:?}", token);
    if let Some(token) = token {
        ft_sdk::println!("session cookie set");
        let (mid, token) = token.split_once(':').expect("token format error");

        if mid == meeting_id.0 {
            ft_sdk::println!("session cookie found for current meeting");
            let session = TalkSession {
                token: Some(token.to_string()),
            };
            return ft_sdk::processor::json(session);
        }
    }

    // talk -> talk.wasm
    let create_new_session_url = app_url.join("talk/session/new")?;
    ft_sdk::processor::temporary_redirect(format!(
        "{create_new_session_url}?meeting-id={meeting_id}"
    ))
}

/// Add the logged in user as participant to the meeting and create a session, or,
/// Use provided name and create guest user if not logged in
#[ft_sdk::form]
fn session_new(
    meeting_id: ft_sdk::Query<"meeting-id", String>,
    user: crate::auth::OptionalUser,
    host: ft_sdk::Host,
    config: crate::Config,
    app_url: ft_sdk::AppUrl,
) -> ft_sdk::form::Result {
    ft_sdk::println!("======= in session new handler ======");
    let (username, name, is_guest) = if user.is_logged_in {
        ft_sdk::println!("Found user name through login");
        (user.username, Some(user.name), false)
    } else {
        let uuid = ft_sdk::uuid();
        ft_sdk::println!("adding guest with id: {uuid}");
        // The frontend will ask for the name
        (uuid, None, true)
    };

    let preset = config.preset_participant;
    let preset = if is_guest {
        // _guest presets are allowed to change their name
        format!("{preset}_guest")
    } else {
        // Name is taken from user's account name and they're not allowed to change it
        preset
    };

    ft_sdk::println!("Using preset: {preset} to create a new session for {username}");

    let username = crate::dyte::Username::new(username, &host);
    let participant =
        crate::dyte::add_participant(&meeting_id, &preset, name.as_deref(), username)?;

    ft_sdk::println!("dyte response: {:?}", participant);

    let session_cookie = crate::create_session_cookie(
        &participant.data.token,
        &meeting_id,
        &host,
        config.secure_sessions,
    )?;

    // lets-talk.fifthtry.site/meeting.ftd
    let meeting_page_url = app_url.join("meeting")?;
    Ok(
        ft_sdk::form::redirect(format!("{meeting_page_url}{meeting_id}/"))?
            .with_cookie(session_cookie),
    )
}

#[derive(serde::Serialize)]
struct TalkSession {
    /// <mid>:<token>
    token: Option<String>,
}
