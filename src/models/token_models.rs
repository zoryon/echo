#[derive(serde::Serialize, serde::Deserialize)]
pub struct Claims {
    pub sub: String, // user ID
    pub exp: i64,    // expiration timestamp
}