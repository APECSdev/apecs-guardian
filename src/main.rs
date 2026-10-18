// src/main.rs

/*!
 * apecs — the APECS Guardian.
 *
 * A read-only, hardened node monitor for top-L1 operators. Registers the host
 * with the APECS control plane, then reports telemetry on an interval. Remote
 * actions are OFF unless `--allow-actions` is passed, and even then only
 * allowlisted, control-plane-signed, unexpired read-only actions can run.
 */

use clap::{Parser, Subcommand};
use human_panic::setup_panic;

#[derive(Parser, Debug)]
#[command(name = "apecs", version, about = "APECS Guardian — read-only node monitor", long_about = None)]
struct Cli {
    /// Control-plane base URL.
    #[arg(long, default_value = apecs::comm::api::DEFAULT_BASE)]
    base: String,

    /// Node API key (x-apecs-key) issued by APECS.
    #[arg(long, env = "APECS_KEY")]
    key: Option<String>,

    /// Telemetry interval, in seconds.
    #[arg(long, default_value_t = 60)]
    interval: u64,

    /// Allow control-plane-signed read-only actions (OFF by default).
    #[arg(long, default_value_t = false)]
    allow_actions: bool,

    #[command(subcommand)]
    command: Option<Cmd>,
}

#[derive(Subcommand, Debug)]
enum Cmd {
    /// Generate a control-plane Ed25519 keypair (control-plane operator only).
    Keygen,
    /// Collect one telemetry snapshot locally and print it (no network).
    Snapshot,
    /// Register with the control plane and start monitoring (default).
    Run,
}

#[tokio::main]
async fn main() {
    setup_panic!();

    let cli = Cli::parse();
    apecs::ui::welcome::banner(apecs::get_version());

    match cli.command {
        Some(Cmd::Keygen) => {
            let keys = apecs::auth::keygen::generate();
            apecs::auth::keygen::print_instructions(&keys);
        }
        Some(Cmd::Snapshot) => {
            let t = apecs::node::telemetry::collect();
            println!("{}", apecs::node::telemetry::to_json(&t));
        }
        Some(Cmd::Run) | None => {
            run(cli).await;
        }
    }
}

/// Register, then enter the monitor loop.
async fn run(cli: Cli) {
    let Some(key) = cli.key.clone() else {
        eprintln!("error: an API key is required. Pass --key or set APECS_KEY.");
        eprintln!("Obtain one from your APECS dashboard.");
        std::process::exit(2);
    };

    /* Register the node. */
    let payload = apecs::node::register::build_payload();
    match apecs::comm::api::post(&cli.base, "register", &payload, &key).await {
        Ok(body) => match apecs::node::register::parse_response(&body) {
            Ok(resp) if resp.success => {
                println!("  Node registered.");
                println!("  Session ID: {}\n", resp.result.sessionid);
                println!("  View live:  https://apecs.dev/guardian/{}\n", resp.result.sessionid);
            }
            _ => {
                eprintln!("error: registration was rejected by the control plane.");
                std::process::exit(1);
            }
        },
        Err(e) => {
            eprintln!("error: could not reach the control plane: {}", e);
            std::process::exit(1);
        }
    }

    /* Monitor loop. */
    println!(
        "  Monitoring every {}s (actions: {}). Ctrl-C to stop.\n",
        cli.interval,
        if cli.allow_actions { "ENABLED" } else { "disabled" }
    );

    let mut ticker = tokio::time::interval(std::time::Duration::from_secs(cli.interval));
    loop {
        ticker.tick().await;

        let snapshot = apecs::node::telemetry::collect();
        let body = apecs::node::telemetry::to_json(&snapshot);

        match apecs::comm::api::post(&cli.base, "telemetry", &body, &key).await {
            Ok(resp_body) => {
                let now_ms = now_unix_ms();
                match serde_json::from_str::<apecs::comm::monitor::PollResponse>(&resp_body) {
                    Ok(poll) => {
                        let outcome =
                            apecs::comm::monitor::handle_poll(&poll, cli.allow_actions, now_ms);
                        use apecs::comm::monitor::Outcome::*;
                        match outcome {
                            NoAction => {}
                            Executed(msg) => println!("  [action] {}", msg),
                            Rejected(why) => println!("  [action rejected] {}", why),
                        }
                    }
                    Err(_) => { /* ack without a parseable action payload; ignore */ }
                }
            }
            Err(e) => eprintln!("  [telemetry] push failed: {}", e),
        }
    }
}

/// Current Unix time in milliseconds.
fn now_unix_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}
