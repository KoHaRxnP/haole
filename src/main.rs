use clap::{Parser, Subcommand};
use colored::*;
use serde::Deserialize;

#[derive(Deserialize)]
struct HavenStatus {
    online: bool,
    players: Players,
    version: String,
}

#[derive(Deserialize)]
struct McStatusIOResponse {
    host: String,
    ip_address: String,
    port: u16,
    version: McStatusIOResponseVersion,
    motd: McStatusIOResponseMotd,
}

#[derive(Deserialize)]
struct McStatusIOResponseVersion {
    protocol: u32,
}

#[derive(Deserialize)]
struct McStatusIOResponseMotd {
    raw: String,
    clean: String,
    html: String,
}

#[derive(Deserialize)]
struct Players {
    online: u32,
    max: u32,
    list: Option<Vec<String>>,
}

#[derive(Parser)]
#[command(name = "haole", about = "HavenMC Status CLI Tool", version = "0.1.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Author,
    Players,
    Pq,
    Pall,
    IsOnline,
    IsOffline,
    Version,
    ServerVersion,
    Ip,
    Host,
    Protocol,
    Port,
    Motd {
        #[arg(short, long)]
        raw: Option<String>,
        clean: Option<String>,
        html: Option<String>,
    },
}

async fn fetch_haven_status() -> Result<HavenStatus, Box<dyn std::error::Error>> {
    let url = "https://api.havenmc.jp/status";
    println!("{} データを取得中...", ">>".blue());
    let resp: HavenStatus = reqwest::get(url).await?.json().await?;
    Ok(resp)
}

async fn fetch_haven_status_by_mcstatusio() -> Result<McStatusIOResponse, Box<dyn std::error::Error>> {
    let url = "https://api.mcstatus.io/v2/status/java/play.havenmc.jp";
    println!("{} データを取得中...", ">>".blue());
    let resp: McStatusIOResponse = reqwest::get(url).await?.json().await?;
    Ok(resp)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli: Cli = Cli::parse();
    let st: HavenStatus = fetch_haven_status().await?;
    let st_mcstatusio: McStatusIOResponse = fetch_haven_status_by_mcstatusio().await?;
    let version: &str = env!("CARGO_PKG_VERSION");

    match &cli.command {
        Commands::Author => {
            println!("Created by: {}", "KoHaRxnP".magenta());
        }
        Commands::Players => {
            if let Some(list) = st.players.list {
                if list.is_empty() {
                    println!("{}", "現在オンラインのプレイヤーはいません。".yellow());
                } else {
                    for player in list {
                        println!(" - {}", player.cyan());
                    }
                }
            } else {
                println!("{}", "プレイヤー名の取得が制限されているか、データがありません。".red());
            }
        }
        Commands::Pq => {
            println!("\n{} {}/{} プレイヤーがオンライン", 
                "●".green(), st.players.online, st.players.max);
        }
        Commands::Pall => {
            if let Some(list) = st.players.list {
                if list.is_empty() {
                    println!("{}", "現在オンラインのプレイヤーはいません。".yellow());
                } else {
                    for player in list {
                        println!(" - {}", player.cyan());
                    }
                }
            } else {
                println!("{}", "プレイヤー名の取得が制限されているか、データがありません。".red());
            }
            println!("\n{} {}/{} プレイヤーがオンライン", 
                "●".green(), st.players.online, st.players.max);
        }
        Commands::IsOnline => {
            if st.online == true {
                println!("{}", "サーバーはオンラインです。".green());
            } else {
                println!("{}", " サーバーはオフラインです。".red());
            }
        }
        Commands::IsOffline => {
            if st.online == false {
                println!("{}", "サーバーはオフラインです。".green());
            } else {
                println!("{}", "サーバーはオンラインです。".red());
            }
        }
        Commands::Version => {
            println!("Haole Version: {}", version.magenta());
        }
        Commands::ServerVersion => {
            println!("Server Version: {}", st.version.magenta());
        }
        Commands::Ip => {
            println!("Server IP: {}", st_mcstatusio.ip_address.magenta());
        }
        Commands::Host => {
            println!("Server Host: {}", st_mcstatusio.host.magenta());
        }
        Commands::Protocol => {
            println!("Protocol Version: {}", st_mcstatusio.version.protocol.to_string().magenta());
        }
        Commands::Port => {
            println!("Server Port: {}", st_mcstatusio.port.to_string().magenta());
        }
        Commands::Motd { raw, clean, html } => {
            if let Some(_query) = raw {
                println!("MOTD (Raw): {}", st_mcstatusio.motd.raw.magenta());
            } else if let Some(_query) = clean {
                println!("MOTD (Clean): {}", st_mcstatusio.motd.clean.magenta());
            } else if let Some(_query) = html {
                println!("MOTD (HTML): {}", st_mcstatusio.motd.html.magenta());
            }
        }
    }

    Ok(())
}