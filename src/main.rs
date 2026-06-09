use clap::Parser;
use rivu_config::loader::ConfigLoader;
use rivu_core::error::Result;
use rivu_core::models::PlayInfo;
use rivu_spider::engine::SpiderApi;
use rivu_spider::site_api::SiteApi;
use rivu_spider::spider::SpiderRegistry;
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

            let mut registry = SpiderRegistry::new();
            registry.register_builtin();
            let engine = SpiderApi::new(SiteApi::new(), registry);

            eprint!("Searching {} sites for '{}'", config.sites.len(), keyword);
            let mut total = 0;
            for site in &config.sites {
                let result = rt.block_on(engine.search(site, &keyword, 1));
                match result {
                    Ok(api_result) => {
                        if let Some(list) = api_result.list {
                            if !list.is_empty() {
                                println!("\n{} {}:", "─".repeat(4), site.name);
                                for vod in &list {
                                    if let Some(ref remarks) = vod.vod_remarks {
                                        println!("  {} [{}]", vod.vod_name, remarks);
                                    } else {
                                        println!("  {}", vod.vod_name);
                                    }
                                }
                                total += list.len();
                            }
                        }
                        eprint!(".");
                    }
                    Err(e) => {
                        eprintln!("\n{} ✗ {}", site.name, e);
                    }
                }
            }
            eprintln!();

            if total == 0 {
                println!("No results found for '{}'", keyword);
            } else {
                println!("\n{} result(s) found for '{}'", total, keyword);
            }
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
