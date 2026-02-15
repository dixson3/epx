use clap::Subcommand;
use std::path::PathBuf;

#[derive(Subcommand, Debug)]
pub enum BookCommand {
    /// Extract an EPUB to a Markdown directory structure
    Extract {
        /// Path to the EPUB file
        file: PathBuf,
        /// Output directory
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Assemble a Markdown directory into an EPUB
    Assemble {
        /// Path to the source directory
        dir: PathBuf,
        /// Output EPUB file path
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Show information about an EPUB file
    Info {
        /// Path to the EPUB file
        file: PathBuf,
    },
    /// Validate an EPUB file
    Validate {
        /// Path to the EPUB file
        file: PathBuf,
    },
}
