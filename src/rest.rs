#[derive(Serialize)]
pub struct VersionsResponse {
    pub versions: Vec<String>,
}

#[derive(Serialize)]
pub struct VersionResponse {
    pub races: Vec<String>,
}
