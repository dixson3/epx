use clap::Subcommand;
use std::path::PathBuf;

#[derive(Subcommand, Debug)]
pub enum AssetCommand {
    /// List assets in an EPUB
    List {
        /// Path to the EPUB file
        file: PathBuf,
        /// Filter by asset type
        #[arg(long, value_parser = ["image", "css", "font", "audio"])]
        r#type: Option<String>,
    },
    /// Extract a single asset
    Extract {
        /// Path to the EPUB file
        file: PathBuf,
        /// Asset path within the EPUB
        asset_path: String,
        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Extract all assets
    ExtractAll {
        /// Path to the EPUB file
        file: PathBuf,
        /// Output directory
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Add an asset to an EPUB
    Add {
        /// Path to the EPUB file
        file: PathBuf,
        /// Path to the asset file to add
        asset: PathBuf,
        /// Media type override
        #[arg(long)]
        media_type: Option<String>,
    },
    /// Remove an asset from an EPUB
    Remove {
        /// Path to the EPUB file
        file: PathBuf,
        /// Asset path within the EPUB
        asset_path: String,
    },
}
