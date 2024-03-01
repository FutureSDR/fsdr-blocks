#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Extension {
    #[serde(rename = "name")]
    pub name: String,
    #[serde(rename = "version")]
    pub version: String,
    #[serde(rename = "optional")]
    pub optional: bool,
}
