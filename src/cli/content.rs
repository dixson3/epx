use clap::Subcommand;
use std::path::PathBuf;

#[derive(Subcommand, Debug)]
pub enum ContentCommand {
    /// Search for text in an EPUB
    Search {
        /// Path to the EPUB file
        file: PathBuf,
        /// Search pattern
        pattern: String,
        /// Limit search to a specific chapter
        #[arg(long)]
        chapter: Option<String>,
        /// Use regex matching
        #[arg(long)]
        regex: bool,
    },
    /// Replace text in an EPUB
    Replace {
        /// Path to the EPUB file
        file: PathBuf,
        /// Search pattern
        pattern: String,
        /// Replacement text
        replacement: String,
        /// Limit to a specific chapter
        #[arg(long)]
        chapter: Option<String>,
        /// Use regex matching
        #[arg(long)]
        regex: bool,
        /// Preview changes without modifying
        #[arg(long)]
        dry_run: bool,
    },
    /// List or restructure headings
    Headings {
        /// Path to the EPUB file
        file: PathBuf,
        /// Heading level mapping (e.g., "h2->h1,h3->h2")
        #[arg(long)]
        restructure: Option<String>,
    },
}
