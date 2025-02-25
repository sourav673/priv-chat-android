use deltachat::provider::Provider;
use num_traits::cast::ToPrimitive;
use serde::Serialize;
use typescript_type_def::TypeDef;

#[derive(Serialize, TypeDef, schemars::JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct ProviderInfo {
    /// Unique ID, corresponding to provider database filename.
    pub id: String,
    pub before_login_hint: String,
    pub overview_page: String,
    pub status: u32, // in reality this is an enum, but for simplicity and because it gets converted into a number anyway, we use an u32 here.
}

impl ProviderInfo {
    pub fn from_dc_type(provider: Option<&Provider>) -> Option<Self> {
        provider.map(|p| ProviderInfo {
            id: p.id.to_owned(),
            before_login_hint: p.before_login_hint.to_owned(),
            overview_page: p.overview_page.to_owned(),
            status: p.status.to_u32().unwrap(),
        })
    }
}
