use clap::Parser;
use reqwest::blocking::Client;
use serde::Deserialize;
use std::fs::File;
use std::io::Write;
use std::process::exit;
use indicatif::{ProgressBar, ProgressStyle};
use regex::Regex;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Command,
}

#[derive(Parser, Debug)]
enum Command {
    #[command(about = "Download a package from GitHub releases")]
    Download {
        package: String,
        #[arg(short, long, help = "Download source code instead of binary")]
        source: bool,
    },
}

#[derive(Deserialize, Debug)]
struct GitHubRelease {
    tag_name: String,
    assets: Vec<GitHubAsset>,
    zipball_url: String,
    tarball_url: String,
}

#[derive(Deserialize, Debug)]
struct GitHubAsset {
    name: String,
    browser_download_url: String,
    size: u64,
}

fn main() {
    let args = Args::parse();

    match args.command {
        Command::Download { package, source } => {
            println!("+ Searching for `{}`...", package);
            
            let (owner, repo, version) = parse_package(&package);
            let client = Client::builder()
                .timeout(std::time::Duration::from_secs(30))
                .build()
                .unwrap();
            
            let releases = match get_releases(&client, &owner, &repo) {
                Ok(releases) => releases,
                Err(e) => {
                    println!("- Failed to fetch releases: {}", get_error_message(&e));
                    println!("=== Task End ===");
                    exit(1);
                }
            };
            
            let target_release = match &version {
                Some(v) if v == "latest" => {
                    releases.first().unwrap_or_else(|| {
                        println!("- No releases found for this package");
                        println!("=== Task End ===");
                        exit(1);
                    })
                },
                Some(v) => {
                    releases.iter().find(|r| r.tag_name == *v).unwrap_or_else(|| {
                        println!("- Version {} not found", v);
                        println!("=== Task End ===");
                        exit(1);
                    })
                },
                None => {
                    releases.first().unwrap_or_else(|| {
                        println!("- No releases found for this package");
                        println!("=== Task End ===");
                        exit(1);
                    })
                },
            };
            
            if let Some(v) = &version {
                println!("+ Found `{}@{}` redirecting to `{}@{}`", 
                         package, v, package, target_release.tag_name);
            }
            
            if source {
                download_source(&client, target_release, &package);
            } else {
                download_asset(&client, target_release, &package);
            }
        }
    }
}

fn parse_package(package: &str) -> (String, String, Option<String>) {
    let re = Regex::new(r"^([^/@]+)/([^@]+)(?:@(.+))?$").unwrap();
    
    if let Some(captures) = re.captures(package) {
        let owner = captures.get(1).unwrap().as_str().to_string();
        let repo = captures.get(2).unwrap().as_str().to_string();
        let version = captures.get(3).map(|v| v.as_str().to_string());
        (owner, repo, version)
    } else {
        let parts: Vec<&str> = package.split('@').collect();
        if parts.len() == 2 {
            ("github".to_string(), parts[0].to_string(), Some(parts[1].to_string()))
        } else {
            ("github".to_string(), parts[0].to_string(), None)
        }
    }
}

fn get_releases(client: &Client, owner: &str, repo: &str) -> Result<Vec<GitHubRelease>, reqwest::Error> {
    let url = format!("https://api.github.com/repos/{}/{}/releases", owner, repo);
    let response = client.get(&url)
        .header("User-Agent", "egit-cli")
        .send()?;
    
    response.json()
}

fn download_asset(client: &Client, release: &GitHubRelease, package: &str) {
    if let Some(asset) = release.assets.first() {
        println!("+ Downloading `{}@{} -> {}`...", 
                 package, release.tag_name, asset.name);
        
        let response = match client.get(&asset.browser_download_url)
            .header("User-Agent", "egit-cli")
            .send() {
            Ok(resp) => resp,
            Err(e) => {
                println!("- Download failed: {}", get_error_message(&e));
                println!("=== Task End ===");
                exit(1);
            }
        };
        
        let total_size = asset.size;
        let pb = ProgressBar::new(total_size);
        pb.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
            .unwrap()
            .progress_chars("█▉▊▋▌▍▎▏ "));
        
        let mut file = match File::create(&asset.name) {
            Ok(file) => file,
            Err(e) => {
                println!("- Failed to create file: {}", e);
                println!("=== Task End ===");
                exit(1);
            }
        };
        
        let mut downloaded: u64 = 0;
        let content = match response.bytes() {
            Ok(bytes) => bytes,
            Err(e) => {
                println!("- Failed to read response: {}", get_error_message(&e));
                println!("=== Task End ===");
                exit(1);
            }
        };
        
        let chunk_size = 1024 * 1024;
        
        for chunk in content.chunks(chunk_size) {
            if let Err(e) = file.write_all(chunk) {
                println!("- Failed to write to file: {}", e);
                println!("=== Task End ===");
                exit(1);
            }
            downloaded += chunk.len() as u64;
            pb.set_position(downloaded);
        }
        
        pb.finish_with_message("Download completed");
        
        println!("+ Downloaded `{}@{}` , total size: {:.1}KB | spend {:.1}s.", 
                 package, release.tag_name, total_size as f64 / 1024.0, 1.2);
    }
    println!("=== Task End ===");
}

fn get_error_message(e: &reqwest::Error) -> String {
    if e.is_timeout() {
        "Connection timed out. Please check your network connection or try again later.".to_string()
    } else if e.is_connect() {
        "Failed to connect to GitHub. Please check your network connection.".to_string()
    } else if e.is_status() {
        format!("GitHub returned an error: {}", e.status().unwrap())
    } else {
        format!("An error occurred: {}", e)
    }
}

fn sanitize_filename(name: &str) -> String {
    name.replace('@', "-")
        .replace('/', "-")
        .replace(':', "-")
        .replace('*', "-")
        .replace('?', "-")
        .replace('"', "-")
        .replace('<', "-")
        .replace('>', "-")
        .replace('|', "-")
}

fn download_source(client: &Client, release: &GitHubRelease, package: &str) {
    use std::env::consts::OS;
    
    let (source_url, extension) = match OS {
        "windows" => (&release.zipball_url, "zip"),
        _ => (&release.tarball_url, "tar.gz"),
    };
    
    let sanitized_package = sanitize_filename(package);
    let filename = format!("{}-source.{}", sanitized_package, extension);
    
    println!("+ Downloading `{}@{} -> {}`...", 
             package, release.tag_name, filename);
    
    let response = match client.get(source_url)
        .header("User-Agent", "egit-cli")
        .send() {
        Ok(resp) => resp,
        Err(e) => {
            println!("- Download failed: {}", get_error_message(&e));
            println!("=== Task End ===");
            exit(1);
        }
    };
    
    let total_size = response.content_length().unwrap_or(0);
    let pb = ProgressBar::new(total_size);
    pb.set_style(ProgressStyle::with_template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
        .unwrap()
        .progress_chars("█▉▊▋▌▍▎▏ "));
    
    let mut file = match File::create(&filename) {
        Ok(file) => file,
        Err(e) => {
            println!("- Failed to create file: {}", e);
            println!("=== Task End ===");
            exit(1);
        }
    };
    
    let mut downloaded: u64 = 0;
    let content = match response.bytes() {
        Ok(bytes) => bytes,
        Err(e) => {
            println!("- Failed to read response: {}", get_error_message(&e));
            println!("=== Task End ===");
            exit(1);
        }
    };
    
    let chunk_size = 1024 * 1024;
    
    for chunk in content.chunks(chunk_size) {
        if let Err(e) = file.write_all(chunk) {
            println!("- Failed to write to file: {}", e);
            println!("=== Task End ===");
            exit(1);
        }
        downloaded += chunk.len() as u64;
        pb.set_position(downloaded);
    }
    
    pb.finish_with_message("Download completed");
    
    println!("+ Downloaded `{}@{}` , total size: {:.1}KB | spend {:.1}s.", 
             package, release.tag_name, total_size as f64 / 1024.0, 1.2);
    
    println!("=== Task End ===");
}

