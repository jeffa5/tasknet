use cookie::CookieJar;
use seed::{prelude::*, *};
use tasknet_shared::cookies::AUTH_PROVIDER_COOKIE;
use tasknet_shared::cookies::SESSION_COOKIE;

#[derive(Debug)]
pub enum Provider {
    /// A public document that requires no sign in, just an id.
    Public,
    /// Sign in with a google account to access a private document.
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
                    "public" => Some(Self::Public),
                    "google" => Some(Self::Google),
                    _ => None,
                })
        } else {
            None
        }
    }

    pub fn logo(&self) -> Node<crate::Msg> {
        match self {
            Self::Public => seed::empty!(),
            Self::Google => seed::img![
                C!["inline", "pr-2"],
                attrs! {At::Src => "/assets/btn_google_light_normal_ios.svg"}
            ],
        }
    }
}

pub fn cookies() -> Option<CookieJar> {
    let cookies_str = html_document().cookie().ok()?;
    let mut jar = cookie::CookieJar::new();

    for cookie_str in cookies_str.split(';') {
        let cookie = cookie::Cookie::parse_encoded(cookie_str).ok()?;
        jar.add(cookie.into_owned());
    }

    let jar_is_empty = jar.iter().next().is_none();
    if jar_is_empty {
        None
    } else {
        Some(jar)
    }
}
