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
#[command(name = "haole", about = "HavenMC Status CLI/TUI Tool", version = env!("CARGO_PKG_VERSION"))]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    #[arg(short, long, global = true)]
    watch: Option<Option<u64>>,
}

#[derive(Subcommand)]
enum Commands {
    #[command(alias = "auth", about = "作者情報を表示します。")]
    Author,
    #[command(alias = "pl", about = "現在オンラインのプレイヤー名を表示します。")]
    Players,
    #[command(about = "現在のプレイヤー数を表示します。")]
    Pq,
    #[command(about = "現在のプレイヤー数とオンラインのプレイヤー名を表示します。")]
    Pall,
    #[command(alias = "isonline", about = "サーバーがオンラインかどうかを確認します。")]
    IsOnline,
    #[command(alias = "isoffline", about = "サーバーがオフラインかどうかを確認します。")]
    IsOffline,
    #[command(about = "Haoleのバージョンを表示します。")]
    Version,
    #[command(alias = "sver", about = "サーバーのバージョンを取得します。")]
    ServerVersion,
    #[command(about = "サーバーのIPアドレスを取得します。")]
    Ip,
    #[command(about = "サーバーのホスト名を取得します。")]
    Host,
    #[command(alias = "proto", about = "サーバーのプロトコルバージョンを取得します。")]
    Protocol,
    #[command(about = "サーバーのポート番号を取得します。")]
    Port,
    #[command(about = "サーバーのMOTDを取得します。")]
    Motd {
        #[arg(short, long)]
        raw: Option<String>,
        clean: Option<String>,
        html: Option<String>,
    },
    #[command(about = "Haoleの動作モードを設定または表示します。")]
    Mode {
        new_mode: Option<String>,
    },
    #[command(about = "Haoleを最新バージョンにアップデートします。")]
    Update,
    #[command(about = "サーバーにPingを送信します。")]
    Ping,
}

async fn fetch_haven_status() -> Result<HavenStatus, Box<dyn std::error::Error>> {
    let url: &str = "https://api.havenmc.jp/status";
    let resp: HavenStatus = reqwest::get(url).await?.json().await?;
    Ok(resp)
}

async fn fetch_haven_status_by_mcstatusio() -> Result<McStatusIOResponse, Box<dyn std::error::Error>> {
    let url: &str = "https://api.mcstatus.io/v2/status/java/play.havenmc.jp";
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
    let default_panic: Box<dyn Fn(&std::panic::PanicHookInfo<'_>) + Send + Sync> = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let _ = crossterm::terminal::disable_raw_mode();
        let _ = execute!(std::io::stdout(), crossterm::terminal::LeaveAlternateScreen);
        default_panic(info);
    }));

    let args: Vec<String> = std::env::args().collect();
    let cfg: HaoleConfig = confy::load("haole", "config").map_err(|e: confy::ConfyError| {
        eprintln!("{} 設定ファイルの読み込みに失敗しました。デフォルト値を使用します: {}", "!!".yellow(), e);
    }).unwrap_or_default();
    if cfg.mode == "tui" && args.len() == 1 {
        run_tui_loop().await?;
        return Ok(());
    }
        let cli: Cli = Cli::parse();
        let interval_secs: u64 = cli.watch
            .map(|inner: Option<u64>| inner.unwrap_or(5))
            .map(|sec: u64| sec.max(2))
            .unwrap_or(0);

        if interval_secs > 0 {
            let mut stdout: std::io::Stdout = stdout();
            
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
    match &cli.command {
        Commands::Author => {
            println!("Created by: {}", "KoHaRxnP".magenta());
            return Ok(());
        }
        Commands::Players => {
            let st: HavenStatus = fetch_haven_status().await?;
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
            return Ok(());
        }
        Commands::Pq => {
            let st: HavenStatus = fetch_haven_status().await?;
            println!("\n{} {}/{} プレイヤーがオンライン", 
                "●".green(), st.players.online, st.players.max);
            return Ok(());
        }
        Commands::Pall => {
            let st: HavenStatus = fetch_haven_status().await?;
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
            return Ok(());
        }
        Commands::IsOnline => {
            let st: HavenStatus = fetch_haven_status().await?;
            if st.online == true {
                println!("{}", "サーバーはオンラインです。".green());
            } else {
                println!("{}", " サーバーはオフラインです。".red());
            }
            return Ok(());
        }
        Commands::IsOffline => {
            let st: HavenStatus = fetch_haven_status().await?;
            if st.online == false {
                println!("{}", "サーバーはオフラインです。".green());
            } else {
                println!("{}", "サーバーはオンラインです。".red());
            }
            return Ok(());
        }
        Commands::Version => {
            let logo: &str = r#"
            _                _      
            | |__   __ _  ___| | ___ 
            | '_ \ / _` |/   \ |/ _ \
            | | | | (_| |  | | |  __/
            |_| |_|\__,_|\___|_|\___|
                "#;
            let version: &str = env!("CARGO_PKG_VERSION");
            println!("{}", logo.green().bold());
            println!("Haole Version: {}", version.magenta());
            return Ok(());
        }
        Commands::ServerVersion => {
            let st: HavenStatus = fetch_haven_status().await?;
            println!("Server Version: {}", st.version.magenta());
            return Ok(());
        }
        Commands::Ip => {
            let st_mcstatusio: McStatusIOResponse = fetch_haven_status_by_mcstatusio().await?;
            println!("Server IP: {}", st_mcstatusio.ip_address.magenta());
            return Ok(());
        }
        Commands::Host => {
            let st_mcstatusio: McStatusIOResponse = fetch_haven_status_by_mcstatusio().await?;
            println!("Server Host: {}", st_mcstatusio.host.magenta());
            return Ok(());
        }
        Commands::Protocol => {
            let st_mcstatusio: McStatusIOResponse = fetch_haven_status_by_mcstatusio().await?;
            println!("Protocol Version: {}", st_mcstatusio.version.protocol.to_string().magenta());
            return Ok(());
        }
        Commands::Port => {
            let st_mcstatusio: McStatusIOResponse = fetch_haven_status_by_mcstatusio().await?;
            println!("Server Port: {}", st_mcstatusio.port.to_string().magenta());
            return Ok(());
        }
        Commands::Motd { raw, clean, html } => {
            let st_mcstatusio: McStatusIOResponse = fetch_haven_status_by_mcstatusio().await?;
            if let Some(_query) = raw {
                println!("MOTD (Raw): {}", st_mcstatusio.motd.raw.magenta());
            } else if let Some(_query) = clean {
                println!("MOTD (Clean): {}", st_mcstatusio.motd.clean.magenta());
            } else if let Some(_query) = html {
                println!("MOTD (HTML): {}", st_mcstatusio.motd.html.magenta());
            } else {
                println!("MOTD: {}", st_mcstatusio.motd.clean.magenta());
            }
            return Ok(());
        }
        Commands::Mode { new_mode } => {
            let mut cfg: HaoleConfig = confy::load("haole", "config")?;
            if let Some(m) = new_mode {
                if m == "cli" || m == "tui" {
                    cfg.mode = m.to_string(); 
                    confy::store("haole", "config", &cfg)?;
                    println!("{} モードを {} に変更しました。", ">>".green(), cfg.mode.cyan());
                } else if m == "toggle" {
                    cfg.mode = if cfg.mode == "cli" { "tui".to_string() } else { "cli".to_string() };
                    confy::store("haole", "config", &cfg)?;
                    println!("{} モードを {} に変更しました。", ">>".green(), cfg.mode.cyan());
                } else {
                    println!("{} 無効なモードです。cli または tui を指定してください。toggleで切り替えることもできます。", "!!".red());
                }
            } else {
                println!("現在のモード: {}", cfg.mode.cyan());
            }
            return Ok(());
        }
        Commands::Update => {
            println!("{} 最新バージョンを確認中...", ">>".blue());

            let handle: tokio::task::JoinHandle<Result<self_update::Status, self_update::errors::Error>> = tokio::task::spawn_blocking(|| {
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
            return Ok(());
        }
        Commands::Ping => {
            not_reccommended();
            under_dev();
            run_ping().await?;
            return Ok(());
        }
    }
}

async fn run_tui_loop() -> Result<(), Box<dyn std::error::Error>> {
    let mut stdout: std::io::Stdout = std::io::stdout();
    execute!(stdout, Clear(ClearType::All), MoveTo(0, 0))?;
    crossterm::terminal::enable_raw_mode()?;
    let mut stdout: std::io::Stdout = std::io::stdout();
    execute!(stdout, crossterm::terminal::EnterAlternateScreen)?;

    let _ = crossterm::terminal::disable_raw_mode();
    let _ = execute!(stdout, crossterm::terminal::LeaveAlternateScreen);
    
    let backend: CrosstermBackend<std::io::Stdout> = CrosstermBackend::new(stdout);
    let mut terminal: Terminal<CrosstermBackend<std::io::Stdout>> = Terminal::new(backend)?;

    terminal.clear()?;

    let mut last_tick: std::time::Instant = std::time::Instant::now();
    let tick_rate: Duration = Duration::from_secs(5);

    let mut st: Option<HavenStatus> = fetch_haven_status().await.ok();

    let mut history: VecDeque<HistoryEntry> = VecDeque::with_capacity(50);

    loop {
        terminal.draw(|f: &mut ratatui::Frame<'_>| {
            let size: ratatui::prelude::Rect = f.area();

            let chunks: std::rc::Rc<[ratatui::prelude::Rect]> = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(0),
                ])
                .split(size);

            let main_layout: std::rc::Rc<[ratatui::prelude::Rect]> = Layout::default()
                .direction(Direction::Horizontal)
                .constraints([
                    Constraint::Percentage(30),
                    Constraint::Percentage(70),
                ])
                .split(chunks[1]);

            let status_text: String = match &st {
                Some(s) => {
                    format!(" サーバー: {} | オンライン: {}/{}", 
                    if s.online { "ONLINE".green() } else { "OFFLINE".red() },
                    s.players.online, s.players.max)
                },
                None => format!(" {} データを取得中、または接続エラー...", "!!".yellow())
            };
            let status_bar: Paragraph<'_> = Paragraph::new(status_text)
                .block(Block::default().borders(Borders::ALL).title(" HavenMC Status "));

            let players_items: Vec<ListItem> = if let Some(ref s) = st {
                s.players.list.as_ref().map_or(vec![], |list: &Vec<String>| {
                    list.iter().map(|p: &String| ListItem::new(format!("  • {}", p))).collect()
                })
            } else { vec![] };
            let players_list: List<'_> = List::new(players_items)
                .block(Block::default().borders(Borders::ALL).title(" Players "));

            let right_chunks: std::rc::Rc<[ratatui::prelude::Rect]> = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(3),
                    Constraint::Min(0),
                ])
                .split(main_layout[1]);

            let data: Vec<u64> = history.iter().map(|e: &HistoryEntry| e.online as u64).collect();
            
            let cmax: u64 = data.iter().max().cloned().unwrap_or(0);
            let max: u64 = if cmax < 10 { 10 } else { cmax + 5 };
            let sparkline: ratatui::widgets::Sparkline<'_> = ratatui::widgets::Sparkline::default()
                .block(Block::default().borders(Borders::LEFT | Borders::RIGHT | Borders::TOP).title(" Activity "))
                .data(&data)
                .max(max)
                .style(ratatui::style::Style::default().fg(ratatui::style::Color::Cyan));

            let history_content: Vec<ListItem> = history.iter().rev()
                .map(|e: &HistoryEntry| ListItem::new(format!(" [{}] {} players", e.time, e.online)))
                .collect();
            let history_list: List<'_> = List::new(history_content)
                .block(Block::default().borders(Borders::LEFT | Borders::RIGHT | Borders::BOTTOM));

            f.render_widget(status_bar, chunks[0]);
            f.render_widget(players_list, main_layout[0]);
            f.render_widget(sparkline, right_chunks[0]);
            f.render_widget(history_list, right_chunks[1]);
        })?;

        let timeout: Duration = tick_rate
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

    crossterm::terminal::disable_raw_mode()?;
    execute!(terminal.backend_mut(), crossterm::terminal::LeaveAlternateScreen)?;
    Ok(())
}

async fn run_ping() -> Result<(), Box<dyn std::error::Error>> {
    println!("{} play.havenmc.jp へ Ping を送信中...", ">>".blue());

    let count_flag: &str = if cfg!(windows) { "-n" } else { "-c" };

    let output = Command::new("ping")
        .args([count_flag, "4", "play.havenmc.jp"])
        .output()
        .map_err(|e| format!("外部コマンド 'ping' の実行に失敗しました。パスが通っているか確認してください: {}", e))?;

    if output.status.success() {
        let stdout: std::borrow::Cow<'_, str> = String::from_utf8_lossy(&output.stdout);
        println!("{}", stdout);
    } else {
        let stderr: std::borrow::Cow<'_, str> = String::from_utf8_lossy(&output.stderr);
        eprintln!("{} Pingに失敗しました: {}", "!!".red(), stderr);
    }

    Ok(())
}