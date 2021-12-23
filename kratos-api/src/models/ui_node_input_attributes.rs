/*
 * Ory Kratos API
 *
 * Documentation for all public and administrative Ory Kratos APIs. Public and administrative APIs are exposed on different ports. Public APIs can face the public internet without any protection while administrative APIs should never be exposed without prior authorization. To protect the administative API port you should use something like Nginx, Ory Oathkeeper, or any other technology capable of authorizing incoming requests. 
 *
 * The version of the OpenAPI document: v0.8.2-alpha.1
 * Contact: hi@ory.sh
 * Generated by: https://openapi-generator.tech
 */

/// UiNodeInputAttributes : InputAttributes represents the attributes of an input node



#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiNodeInputAttributes {
    /// Sets the input's disabled field to true or false.
    #[serde(rename = "disabled")]
    pub disabled: bool,
    #[serde(rename = "label", skip_serializing_if = "Option::is_none")]
    pub label: Option<Box<crate::models::UiText>>,
    /// The input's element name.
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "node_type")]
    pub node_type: String,
    /// OnClick may contain javascript which should be executed on click. This is primarily used for WebAuthn.
    #[serde(rename = "onclick", skip_serializing_if = "Option::is_none")]
    pub onclick: Option<String>,
    /// The input's pattern.
    #[serde(rename = "pattern", skip_serializing_if = "Option::is_none")]
    pub pattern: Option<String>,
    /// Mark this input field as required.
    #[serde(rename = "required", skip_serializing_if = "Option::is_none")]
    pub required: Option<bool>,
    #[serde(rename = "type")]
    pub _type: String,
    /// The input's value.
    #[serde(rename = "value", skip_serializing_if = "Option::is_none")]
    pub value: Option<serde_json::Value>,
}

impl UiNodeInputAttributes {
    /// InputAttributes represents the attributes of an input node
    pub fn new(disabled: bool, name: String, node_type: String, _type: String) -> UiNodeInputAttributes {
        UiNodeInputAttributes {
            disabled,
            label: None,
            name,
            node_type,
            onclick: None,
            pattern: None,
            required: None,
            _type,
            value: None,
        }
    }
}


