use crate::epub::writer;
use std::path::Path;

/// Assemble a directory into an EPUB file
pub fn package_epub(dir: &Path, output: &Path) -> anyhow::Result<()> {
    let book = super::assemble_book(dir)?;
    writer::write_epub(&book, output)?;
    Ok(())
}
