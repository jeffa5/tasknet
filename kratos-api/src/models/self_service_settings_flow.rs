/*
 * Ory Kratos API
 *
 * Documentation for all public and administrative Ory Kratos APIs. Public and administrative APIs are exposed on different ports. Public APIs can face the public internet without any protection while administrative APIs should never be exposed without prior authorization. To protect the administative API port you should use something like Nginx, Ory Oathkeeper, or any other technology capable of authorizing incoming requests. 
 *
 * The version of the OpenAPI document: v0.8.2-alpha.1
 * Contact: hi@ory.sh
 * Generated by: https://openapi-generator.tech
 */

/// SelfServiceSettingsFlow : This flow is used when an identity wants to update settings (e.g. profile data, passwords, ...) in a selfservice manner.  We recommend reading the [User Settings Documentation](../self-service/flows/user-settings)



#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SelfServiceSettingsFlow {
    /// Active, if set, contains the registration method that is being used. It is initially not set.
    #[serde(rename = "active", skip_serializing_if = "Option::is_none")]
    pub active: Option<String>,
    /// ExpiresAt is the time (UTC) when the flow expires. If the user still wishes to update the setting, a new flow has to be initiated.
    #[serde(rename = "expires_at")]
    pub expires_at: String,
    #[serde(rename = "id")]
    pub id: String,
    #[serde(rename = "identity")]
    pub identity: Box<crate::models::Identity>,
    /// IssuedAt is the time (UTC) when the flow occurred.
    #[serde(rename = "issued_at")]
    pub issued_at: String,
    /// RequestURL is the initial URL that was requested from Ory Kratos. It can be used to forward information contained in the URL's path or query for example.
    #[serde(rename = "request_url")]
    pub request_url: String,
    /// ReturnTo contains the requested return_to URL.
    #[serde(rename = "return_to", skip_serializing_if = "Option::is_none")]
    pub return_to: Option<String>,
    #[serde(rename = "state")]
    pub state: crate::models::SelfServiceSettingsFlowState,
    /// The flow type can either be `api` or `browser`.
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub _type: Option<String>,
    #[serde(rename = "ui")]
    pub ui: Box<crate::models::UiContainer>,
}

impl SelfServiceSettingsFlow {
    /// This flow is used when an identity wants to update settings (e.g. profile data, passwords, ...) in a selfservice manner.  We recommend reading the [User Settings Documentation](../self-service/flows/user-settings)
    pub fn new(expires_at: String, id: String, identity: crate::models::Identity, issued_at: String, request_url: String, state: crate::models::SelfServiceSettingsFlowState, ui: crate::models::UiContainer) -> SelfServiceSettingsFlow {
        SelfServiceSettingsFlow {
            active: None,
            expires_at,
            id,
            identity: Box::new(identity),
            issued_at,
            request_url,
            return_to: None,
            state,
            _type: None,
            ui: Box::new(ui),
        }
    }
}

