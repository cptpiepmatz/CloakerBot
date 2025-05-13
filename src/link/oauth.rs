use std::sync::LazyLock;

use url::Url;

pub const LINKED_ROLE_PATH: &str = "/linked-role";
pub async fn linked_role() {}

pub const CALLBACK_PATH: &str = "/callback";
pub async fn callback() {}

pub static OAUTH_URL: LazyLock<Url> = LazyLock::new(|| {
    let mut url = Url::parse("https://discord.com/api/oauth2/authorize").expect("valid url");
    // url.query_pairs_mut().append_pair("client_id", value)

    todo!()
});
