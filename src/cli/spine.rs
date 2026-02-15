use clap::Subcommand;
use std::path::PathBuf;

#[derive(Subcommand, Debug)]
pub enum SpineCommand {
    /// List spine items
    List {
        /// Path to the EPUB file
        file: PathBuf,
    },
    /// Reorder a spine item
    Reorder {
        /// Path to the EPUB file
        file: PathBuf,
        /// Current position (index)
        from: usize,
        /// New position (index)
        to: usize,
    },
    /// Set spine order from a YAML file
    Set {
        /// Path to the EPUB file
        file: PathBuf,
        /// Path to the spine YAML file
        spine: PathBuf,
    },
}
