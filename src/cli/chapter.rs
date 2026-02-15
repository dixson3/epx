use clap::Subcommand;
use std::path::PathBuf;

#[derive(Subcommand, Debug)]
pub enum ChapterCommand {
    /// List chapters in an EPUB
    List {
        /// Path to the EPUB file
        file: PathBuf,
    },
    /// Extract a single chapter to Markdown
    Extract {
        /// Path to the EPUB file
        file: PathBuf,
        /// Chapter ID or index
        id: String,
        /// Output file path
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// Add a Markdown chapter to an EPUB
    Add {
        /// Path to the EPUB file
        file: PathBuf,
        /// Path to the Markdown file
        markdown: PathBuf,
        /// Insert after this chapter ID or index
        #[arg(long)]
        after: Option<String>,
        /// Chapter title
        #[arg(long)]
        title: Option<String>,
    },
    /// Remove a chapter from an EPUB
    Remove {
        /// Path to the EPUB file
        file: PathBuf,
        /// Chapter ID or index
        id: String,
    },
    /// Reorder a chapter in an EPUB
    Reorder {
        /// Path to the EPUB file
        file: PathBuf,
        /// Current position (index)
        from: usize,
        /// New position (index)
        to: usize,
    },
}
