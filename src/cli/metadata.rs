use clap::Subcommand;
use std::path::PathBuf;

#[derive(Subcommand, Debug)]
pub enum MetadataCommand {
    /// Show EPUB metadata
    Show {
        /// Path to the EPUB file
        file: PathBuf,
    },
    /// Set a metadata field
    Set {
        /// Path to the EPUB file
        file: PathBuf,
        /// Metadata field name
        #[arg(long)]
        field: String,
        /// Metadata field value
        #[arg(long)]
        value: String,
    },
    /// Remove a metadata field
    Remove {
        /// Path to the EPUB file
        file: PathBuf,
        /// Metadata field name
        #[arg(long)]
        field: String,
    },
    /// Import metadata from a YAML file
    Import {
        /// Path to the EPUB file
        file: PathBuf,
        /// Path to the YAML metadata file
        metadata: PathBuf,
    },
    /// Export metadata to a YAML file
    Export {
        /// Path to the EPUB file
        file: PathBuf,
        /// Output YAML file path
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
}
