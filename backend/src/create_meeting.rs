#[ft_sdk::form]
/// Create a meeting and add the user to it as a participant
fn create_meeting(
    title: ft_sdk::Required<"title">,
    user: crate::auth::RequiredUser,
    host: ft_sdk::Host,
    config: crate::Config,
    app_url: ft_sdk::AppUrl,
    lets_auth_app_url: ft_sdk::AppUrl<"lets-auth">,
) -> ft_sdk::form::Result {
    if let Err(e) = user.is_special(&config, &host) {
        use crate::auth::AuthorizationError;
        let msg = match e {
            AuthorizationError::Unauthorized => {
                "You are not authorized to create a meeting. Please contact the admin.".to_string()
            }
            AuthorizationError::EmptyWhitelist => {
                "`who-can-create-meetings` variable is empty. You can't create a meeting."
                    .to_string()
            }
            AuthorizationError::RequiresVerification => {
                let verification_link = lets_auth_app_url.join(
                    format!(
                        "/backend/resend-confirmation-email/?email={email}",
                        email = user.email
                    ),
                );
                let link_text = match verification_link {
                    Ok(v) => format!(
                        " or [click here]({}) to get a new email",
                        v.trim_end_matches('/')
                    ),
                    Err(e) => {
                        ft_sdk::println!("Error creating verification link: {e}");
                        "".to_string()
                    }
                };
                format!(
                    "Verify your email to create a meeting. Check your email{link_text}.",
                )
            }
        };

        return Err(title.error(msg).into());
    }

    let meeting = crate::dyte::create_meeting(&title)?;

    let name = if user.name.is_empty() {
        None
    } else {
        Some(user.name.as_str())
    };

    let preset = config.preset_host;

    ft_sdk::println!("Using preset: {preset} to create meeting");

    let username = crate::dyte::Username::new(user.username, &host);
    let participant = crate::dyte::add_participant(&meeting.data.id, &preset, name, username)?;

    let session_cookie = crate::create_session_cookie(
        &participant.data.token,
        &meeting.data.id,
        &host,
        config.secure_sessions,
    )?;

    // lets-talk.fifthtry.site/meeting.ftd
    let meeting_page_url = app_url.join("meeting")?;

    Ok(
        ft_sdk::form::redirect(format!("{meeting_page_url}{}", meeting.data.id))?
            .with_cookie(session_cookie),
    )
}
