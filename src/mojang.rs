use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct MojangProfile {
    pub id: String,
    pub name: String,
}

pub async fn fetch_profile(client: &reqwest::Client, name: &str) -> Result<Option<MojangProfile>, Box<dyn std::error::Error + Send + Sync>> {
    let url = format!("https://api.mojang.com/users/profiles/minecraft/{}", name);
    let resp = client.get(&url).send().await?;

    if resp.status().is_success() {
        let profile = resp.json::<MojangProfile>().await?;
        Ok(Some(profile))
    } else if resp.status().as_u16() == 404 {
        Ok(None)
    } else {
        Err(format!("Mojang API returned error: {}", resp.status()).into())
    }
}
