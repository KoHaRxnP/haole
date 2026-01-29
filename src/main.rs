use clap::{Parser, Subcommand};
use colored::*;
use std::collections::VecDeque;
use std::process::Command;
use serde::Deserialize;
use tokio::time::{Duration};
use std::io::{stdout};
use chrono::Local;
use crossterm::{
    execute,
    terminal::{Clear, ClearType},
    cursor::MoveTo,
    event::{self, Event, KeyCode, KeyEventKind},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Layout, Constraint, Direction},
    widgets::{Block, Borders, Paragraph, List, ListItem},
    Terminal,
};

#[derive(serde::Serialize, serde::Deserialize, Debug)]
struct HaoleConfig {
    mode: String,
}

impl Default for HaoleConfig {
    fn default() -> Self {
        Self { mode: "cli".into() }
    }
}

struct HistoryEntry {
    time: String,
    online: u32,
}

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
#[command(name = "haole", about = "HavenMC Status CLI Tool", version = env!("CARGO_PKG_VERSION"))]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, global = true)]
    watch: Option<Option<u64>>,
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
    Mode {
        new_mode: Option<String>,
    },
    Update,
    Ping,
}

async fn fetch_haven_status() -> Result<HavenStatus, Box<dyn std::error::Error>> {
    let url = "https://api.havenmc.jp/status";
    let resp: HavenStatus = reqwest::get(url).await?.json().await?;
    Ok(resp)
}

async fn fetch_haven_status_by_mcstatusio() -> Result<McStatusIOResponse, Box<dyn std::error::Error>> {
    let url = "https://api.mcstatus.io/v2/status/java/play.havenmc.jp";
    let resp: McStatusIOResponse = reqwest::get(url).await?.json().await?;
    Ok(resp)
}

fn not_reccommended() {
    println!("{}", "!! このコマンドは現在推奨されていません。不安定な動作をする、もしくは機能しない可能性があります。\n".yellow());
}

fn under_dev() {
    println!("{}", "!! この機能は現在開発中です。不安定な動作をする、もしくは機能しない可能性があります。\n".yellow());
}
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    let cfg: HaoleConfig = confy::load("haole", "config").unwrap_or_default();
    if cfg.mode == "tui" && args.len() == 1 {
        run_tui_loop().await?;
        return Ok(());
    }
        let cli: Cli = Cli::parse();
        let interval_secs = match cli.watch {
            Some(Some(sec)) => if sec < 2 { 2 } else { sec },
            Some(None) => 5,
            None => 0,
        };

        if interval_secs > 0 {
            let mut stdout = stdout();
            
            loop {
                execute!(stdout, Clear(ClearType::All), MoveTo(0, 0))?;
                println!("{} {}秒おきに監視中... (Qキーで終了)\n", ">>".blue(), interval_secs);

                if let Err(e) = run_app(&cli).await {
                    println!("{} エラー: {}", "!!".red(), e);
                }

                if event::poll(Duration::from_secs(interval_secs))? {
                    if let Event::Key(key) = event::read()? {
                        if key.kind == KeyEventKind::Press {
                            if key.code == KeyCode::Char('q') || key.code == KeyCode::Char('Q') {
                                println!("\n{}", "監視を終了しました。".yellow());
                                break;
                            }
                        }
                    }
                }
            }
        } else {
            run_app(&cli).await?;
        }

    Ok(())
}

async fn run_app(cli: &Cli) -> Result<(), Box<dyn std::error::Error>> {
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
            let logo = r#"
            _                _      
            | |__   __ _  ___| | ___ 
            | '_ \ / _` |/   \ |/ _ \
            | | | | (_| |  | | |  __/
            |_| |_|\__,_|\___|_|\___|
                "#;
            println!("{}", logo.green().bold());
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
            } else {
                println!("MOTD: {}", st_mcstatusio.motd.clean.magenta());
            }
        }
        Commands::Mode { new_mode } => {
            let mut cfg: HaoleConfig = confy::load("haole", "config")?;
            if let Some(m) = new_mode {
                if m == "cli" || m == "tui" {
                    cfg.mode = m.to_string(); 
                    confy::store("haole", "config", &cfg)?;
                    println!("{} モードを {} に変更しました。", ">>".green(), cfg.mode.cyan());
                } else {
                    println!("{} 無効なモードです。cli または tui を指定してください。", "!!".red());
                }
            } else {
                println!("現在のモード: {}", cfg.mode.cyan());
            }
        }
        Commands::Update => {
            println!("{} 最新バージョンを確認中...", ">>".blue());

            let handle = tokio::task::spawn_blocking(|| {
                self_update::backends::github::Update::configure()
                    .repo_owner("KoHaRxnP")
                    .repo_name("haole")
                    .bin_name("haole")
                    .show_download_progress(true)
                    .current_version(env!("CARGO_PKG_VERSION"))
                    .build()
                    .and_then(|update| update.update())
            });

            match handle.await? {
                Ok(status) => {
                    if status.updated() {
                        println!("{} アップデートが完了しました！ ({})", ">>".green(), status.version());
                    } else {
                        println!("{} すでに最新バージョン ({}) です。", ">>".yellow(), status.version());
                    }
                }
                Err(e) => println!("{} アップデート中にエラーが発生しました: {}", "!!".red(), e),
            }
        }
        Commands::Ping => {
            not_reccommended();
            under_dev();
            run_ping().await?;
        }
    }
    Ok(())
}

async fn run_tui_loop() -> Result<(), Box<dyn std::error::Error>> {
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut last_tick = std::time::Instant::now();
    let tick_rate = Duration::from_secs(5);

    let mut st = fetch_haven_status().await.ok();

    let mut history: VecDeque<HistoryEntry> = VecDeque::with_capacity(50);

    loop {
        terminal.draw(|f| {
            let size = f.size();

            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(0),
                ])
                .split(size);

            let main_layout = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(30),
                    Constraint::Percentage(70),
                ])
                .split(chunks[1]);

            let status_text = if let Some(ref s) = st {
                format!(" サーバー: {} | オンライン: {}/{}", 
                    if s.online { "ONLINE".green() } else { "OFFLINE".red() },
                    s.players.online, s.players.max)
            } else {
                "データを取得中...".into()
            };
            let status_bar = Paragraph::new(status_text)
                .block(Block::default().borders(Borders::ALL).title(" HavenMC Status "));

            let players_items: Vec<ListItem> = if let Some(ref s) = st {
                s.players.list.as_ref().map_or(vec![], |list| {
                    list.iter().map(|p| ListItem::new(format!("  • {}", p))).collect()
                })
            } else { vec![] };
            let players_list = List::new(players_items)
                .block(Block::default().borders(Borders::ALL).title(" Players "));

            let right_chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(0),
                ])
                .split(main_layout[1]);

            let data: Vec<u64> = history.iter().map(|e| e.online as u64).collect();
            
            let cmax = data.iter().max().cloned().unwrap_or(0);
            let max = if cmax < 10 { 10 } else { cmax + 5 };
            let sparkline = ratatui::widgets::Sparkline::default()
                .block(Block::default().borders(Borders::LEFT | Borders::RIGHT | Borders::TOP).title(" Activity "))
                .data(&data)
                .max(max)
                .style(ratatui::style::Style::default().fg(ratatui::style::Color::Cyan));

            let history_content: Vec<ListItem> = history.iter().rev()
                .map(|e| ListItem::new(format!(" [{}] {} players", e.time, e.online)))
                .collect();
            let history_list = List::new(history_content)
                .block(Block::default().borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM));

            f.render_widget(status_bar, chunks[0]);
            f.render_widget(players_list, main_layout[0]);
            f.render_widget(sparkline, right_chunks[0]);
            f.render_widget(history_list, right_chunks[1]);
        })?;

        let timeout = tick_rate
            .checked_sub(last_tick.elapsed())
            .unwrap_or(Duration::from_secs(0));

        if crossterm::event::poll(timeout)? {
            if let Event::Key(key) = crossterm::event::read()? {
                if key.kind == KeyEventKind::Press {
                    if key.code == KeyCode::Char('q') {
                        break;
                    }
                }
            }
        }

        if last_tick.elapsed() >= tick_rate {
            if last_tick.elapsed() >= tick_rate {
            st = fetch_haven_status().await.ok();

            if let Some(ref s) = st {
                if history.len() >= 50 {
                    history.pop_front();
                }
                history.push_back(HistoryEntry {
                    time: Local::now().format("%H:%M:%S").to_string(),
                    online: s.players.online,
                });
            }
            last_tick = std::time::Instant::now();
        }
        }
    }

    crossterm::terminal::disable_raw_mode()?;
    execute!(terminal.backend_mut(), crossterm::terminal::LeaveAlternateScreen)?;
    Ok(())
}

async fn run_ping() -> Result<(), Box<dyn std::error::Error>> {
    println!("{} play.havenmc.jp へ Ping を送信中...", ">>".blue());

    let count_flag = if cfg!(windows) { "-n" } else { "-c" };

    let output = Command::new("ping")
        .args([count_flag, "4", "play.havenmc.jp"])
        .output()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("{}", stdout);
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        eprintln!("{} Pingに失敗しました: {}", "!!".red(), stderr);
    }

    Ok(())
}