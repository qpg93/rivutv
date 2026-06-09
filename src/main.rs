use clap::Parser;
use rivu_config::loader::ConfigLoader;
use rivu_core::error::Result;
use rivu_core::models::PlayInfo;
use rivu_ui::app::App;
use std::collections::HashMap;

#[derive(Parser)]
#[command(name = "rivu")]
#[command(version, about = "RivuTV - A Linux-native TVBox media client")]
enum Cli {
    /// Launch the interactive TUI
    Run,
    /// Configure a source URL
    Config {
        /// TVBox source JSON URL
        url: String,
    },
    /// List configured sources
    Sources,
    /// Search media across all sources
    Search {
        keyword: String,
    },
    /// Play a URL directly
    Play {
        url: String,
    },
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli {
        Cli::Run => {
            let config_dir = ConfigLoader::get_config_dir();
            let mut loader = ConfigLoader::new(&config_dir);
            let mut app = App::new();

            let source_url = loader.app_config.source_url.clone();
            if let Some(url) = source_url {
                match loader.fetch_source(&url).await {
                    Ok(config) => {
                        app.set_sites(config.sites.clone());
                    }
                    Err(e) => {
                        eprintln!("Warning: couldn't load source config: {}", e);
                    }
                }
            }

            app.run()?;
        }
        Cli::Config { url } => {
            let config_dir = ConfigLoader::get_config_dir();
            let mut loader = ConfigLoader::new(&config_dir);
            loader.app_config.source_url = Some(url.clone());
            loader.save_app_config()?;
            println!("Source URL saved: {}", url);
        }
        Cli::Sources => {
            let config_dir = ConfigLoader::get_config_dir();
            let loader = ConfigLoader::new(&config_dir);
            if let Some(ref url) = loader.app_config.source_url {
                println!("Configured source: {}", url);
            } else {
                println!("No source configured. Use: rivu config <url>");
            }
        }
        Cli::Search { keyword } => {
            println!("Search for '{}' (not yet implemented in CLI mode)", keyword);
        }
        Cli::Play { url } => {
            let player = rivu_player::MpvBackend::new();
            let info = PlayInfo {
                url,
                headers: HashMap::new(),
                user_agent: None,
                referer: None,
            };
            player.play(&info)?;
            println!("Press Ctrl+C to stop playback...");
            tokio::signal::ctrl_c().await?;
            player.stop()?;
        }
    }

    Ok(())
}
