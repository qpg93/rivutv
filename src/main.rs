use clap::Parser;

#[derive(Parser)]
#[command(name = "rivu")]
#[command(version, about = "RivuTV - A Linux-native TVBox media client")]
enum Cli {
    /// Launch the interactive UI
    Run,
    /// List configured sources
    Sources,
    /// Search media across all sources
    Search { keyword: String },
    /// Play a URL directly
    Play { url: String },
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    match cli {
        Cli::Run => {
            let app = rivu_ui::app::App::new();
            app.run();
        }
        Cli::Sources => {
            println!("Sources: (not yet implemented)");
        }
        Cli::Search { keyword } => {
            println!("Searching for: {keyword}");
        }
        Cli::Play { url } => {
            let player = rivu_player::backends::MpvBackend::new();
            player.play(&url);
        }
    }
}
