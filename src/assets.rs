use reqwest::blocking::Client;
use serde::Deserialize;
use std::fmt;

#[derive(Deserialize, Debug)]
pub struct GitHubTag {
    pub name: String,
}

#[derive(Deserialize, Debug)]
pub struct GitHubRelease {
    pub tag_name: String,
    pub name: Option<String>,
    pub published_at: Option<String>,
    pub assets: Vec<GitHubAsset>,
}

#[derive(Deserialize, Debug)]
pub struct GitHubAsset {
    pub name: String,
    pub browser_download_url: String,
    pub size: u64,
}

impl fmt::Display for GitHubTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl fmt::Display for GitHubAsset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let size_kb = self.size as f64 / 1024.0;
        write!(f, "- {} ({:.1} KB)\n  URL: {}", 
               self.name, size_kb, self.browser_download_url)
    }
}

impl fmt::Display for GitHubRelease {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = self.name.as_deref().unwrap_or("Unnamed release");
        let date = self.published_at.as_deref().unwrap_or("Unknown date");
        write!(f, "{} - {} (published: {})
  Assets: {}", 
               self.tag_name, name, date, self.assets.len())
    }
}

pub fn display_assets(release: &GitHubRelease) {
    println!("=== Assets for Release '{}' ===", release.tag_name);
    if release.assets.is_empty() {
        println!("- No assets found for this release");
    } else {
        for asset in &release.assets {
            println!("{}", asset);
        }
    }
    println!("=== Total: {} assets ===", release.assets.len());
}

pub fn fetch_tags(client: &Client, owner: &str, repo: &str) -> Result<Vec<GitHubTag>, reqwest::Error> {
    let url = format!("https://api.github.com/repos/{}/{}/tags", owner, repo);
    client.get(&url)
        .header("User-Agent", "egit-cli")
        .send()?
        .json()
}

pub fn fetch_releases(client: &Client, owner: &str, repo: &str) -> Result<Vec<GitHubRelease>, reqwest::Error> {
    let url = format!("https://api.github.com/repos/{}/{}/releases", owner, repo);
    client.get(&url)
        .header("User-Agent", "egit-cli")
        .send()?
        .json()
}

pub fn display_tags(tags: &[GitHubTag]) {
    println!("=== Tags ===");
    for tag in tags {
        println!("- {}", tag);
    }
    println!("=== Total: {} tags ===", tags.len());
}

pub fn display_releases(releases: &[GitHubRelease]) {
    println!("=== Releases ===");
    for release in releases {
        println!("- {}", release);
    }
    println!("=== Total: {} releases ===", releases.len());
}
