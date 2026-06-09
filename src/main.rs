use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use clap::Parser;
use rivu_config::loader::ConfigLoader;
use rivu_core::error::Result;
use rivu_core::models::PlayInfo;
use rivu_spider::engine::SpiderApi;
use rivu_spider::site_api::SiteApi;
use rivu_spider::spider::SpiderRegistry;
use rivu_ui::app::App;

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

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli {
        Cli::Run => {
            let config_dir = ConfigLoader::get_config_dir();
            let mut loader = ConfigLoader::new(&config_dir);
            let mut app = App::new();

            let source_url = loader.app_config.source_url.clone();
            if let Some(url) = source_url {
                let rt = tokio::runtime::Runtime::new()?;
                match rt.block_on(loader.fetch_source(&url)) {
                    Ok(config) => {
                        app.set_sites(config.sites.clone());
                        app.load_home();
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
            let config_dir = ConfigLoader::get_config_dir();
            let mut loader = ConfigLoader::new(&config_dir);

            let source_url = match loader.app_config.source_url.clone() {
                Some(url) => url,
                None => {
                    eprintln!("No source configured. Use: rivu config <url>");
                    return Ok(());
                }
            };

            let rt = tokio::runtime::Runtime::new()?;
            let config = match rt.block_on(loader.fetch_source(&source_url)) {
                Ok(c) => c,
                Err(e) => {
                    eprintln!("Error loading source: {}", e);
                    return Ok(());
                }
            };

            let engine = Arc::new(SpiderApi::new(SiteApi::new(), SpiderRegistry::new()));

            let mut app = App::new();
            app.set_sites(config.sites.clone());
            app.search.query = keyword.clone();

            eprint!("Searching {} sites for '{}'", config.sites.len(), keyword);
            let mut handles = Vec::new();
            for site in &config.sites {
                let e = engine.clone();
                let s = site.clone();
                let kw = keyword.clone();
                handles.push(tokio::spawn(async move {
                    let result = tokio::time::timeout(Duration::from_secs(10), e.search(&s, &kw, 1)).await;
                    (s.name, result)
                }));
            }

            for handle in handles {
                if let Ok((_name, Ok(Ok(api_result)))) = rt.block_on(handle) {
                    if let Some(list) = api_result.list {
                        for vod in list {
                            app.search.result_sites.push(_name.clone());
                            app.search.results.push(vod);
                        }
                    }
                    eprint!(".");
                } else {
                    eprint!(".");
                }
            }
            eprintln!();

            if app.search.results.is_empty() {
                println!("No results found for '{}'", keyword);
                return Ok(());
            }

            println!("{} result(s) found. Opening TUI...", app.search.results.len());
            app.current = rivu_ui::app::Screen::Search;
            app.run()?;
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
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(tokio::signal::ctrl_c())?;
            player.stop()?;
        }
    }

    Ok(())
}
