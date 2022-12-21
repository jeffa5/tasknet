use seed::cookies;

pub const SESSION_COOKIE: &str = "session";
pub const AUTH_PROVIDER_COOKIE: &str = "auth-provider";

pub fn provider() -> Option<String> {
    let have_session = cookies()
        .map(|cookie_jar| cookie_jar.get(SESSION_COOKIE).is_some())
        .unwrap_or_default();

    if have_session {
        cookies().and_then(|cookie_jar| {
            cookie_jar
                .get(AUTH_PROVIDER_COOKIE)
                .map(|cookie| cookie.value())
                .map(std::borrow::ToOwned::to_owned)
        })
    } else {
        None
    }
}
