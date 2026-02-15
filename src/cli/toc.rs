use clap::Subcommand;
use std::path::PathBuf;

#[derive(Subcommand, Debug)]
pub enum TocCommand {
    /// Show the table of contents
    Show {
        /// Path to the EPUB file
        file: PathBuf,
        /// Maximum depth to display
        #[arg(long)]
        depth: Option<usize>,
    },
    /// Set the table of contents from a Markdown file
    Set {
        /// Path to the EPUB file
        file: PathBuf,
        /// Path to the TOC Markdown file
        toc: PathBuf,
    },
    /// Generate a table of contents from headings
    Generate {
        /// Path to the EPUB file
        file: PathBuf,
        /// Maximum heading depth to include
        #[arg(long)]
        depth: Option<usize>,
    },
}
