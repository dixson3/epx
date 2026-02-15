pub mod asset;
pub mod book;
pub mod chapter;
pub mod content;
pub mod metadata;
pub mod output;
pub mod spine;
pub mod toc;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "epx", version, about = "Extract, manipulate, and assemble EPUB files")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Resource,

    /// Output as JSON
    #[arg(long, global = true)]
    pub json: bool,

    /// Verbose output
    #[arg(long, short, global = true)]
    pub verbose: bool,

    /// Suppress non-essential output
    #[arg(long, short, global = true)]
    pub quiet: bool,

    /// Disable colored output
    #[arg(long, global = true, env = "NO_COLOR")]
    pub no_color: bool,
}

#[derive(Subcommand, Debug)]
pub enum Resource {
    /// Whole-book operations: extract, assemble, info, validate
    Book {
        #[command(subcommand)]
        command: book::BookCommand,
    },
    /// Chapter operations: list, extract, add, remove, reorder
    Chapter {
        #[command(subcommand)]
        command: chapter::ChapterCommand,
    },
    /// Metadata operations: show, set, remove, import, export
    Metadata {
        #[command(subcommand)]
        command: metadata::MetadataCommand,
    },
    /// Table of contents: show, set, generate
    Toc {
        #[command(subcommand)]
        command: toc::TocCommand,
    },
    /// Spine operations: list, reorder, set
    Spine {
        #[command(subcommand)]
        command: spine::SpineCommand,
    },
    /// Asset operations: list, extract, extract-all, add, remove
    Asset {
        #[command(subcommand)]
        command: asset::AssetCommand,
    },
    /// Content operations: search, replace, headings
    Content {
        #[command(subcommand)]
        command: content::ContentCommand,
    },
}
