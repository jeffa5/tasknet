use seed::cookies;
use seed::{prelude::*, *};

pub const SESSION_COOKIE: &str = "session";
pub const AUTH_PROVIDER_COOKIE: &str = "auth-provider";

#[derive(Debug)]
pub enum Provider {
    Google,
}

impl Provider {
    pub fn load_from_session() -> Option<Self> {
        let have_session = cookies()
            .map(|cookie_jar| cookie_jar.get(SESSION_COOKIE).is_some())
            .unwrap_or_default();

        if have_session {
            cookies()
                .and_then(|cookie_jar| {
                    cookie_jar
                        .get(AUTH_PROVIDER_COOKIE)
                        .map(|cookie| cookie.value())
                        .map(ToOwned::to_owned)
                })
                .and_then(|provider| match provider.as_str() {
                    "google" => Some(Self::Google),
                    _ => None,
                })
        } else {
            None
        }
    }

    pub fn logo(&self) -> Node<crate::Msg> {
        match self {
            Provider::Google => seed::img![
                C!["inline", "pr-2"],
                attrs! {At::Src => "/assets/btn_google_light_normal_ios.svg"}
            ],
        }
    }
}
