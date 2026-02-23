mod assemble;
mod cli;
mod epub;
mod error;
mod extract;
mod manipulate;
mod util;

use anyhow::{Context, Result};
use clap::Parser;
use cli::{Cli, Resource};

/// Format a byte count as a human-readable size string.
fn format_size(bytes: usize) -> String {
    const KB: usize = 1024;
    const MB: usize = 1024 * KB;
    if bytes >= MB {
        format!("{:.1} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.1} KB", bytes as f64 / KB as f64)
    } else {
        format!("{bytes} B")
    }
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let output =
        cli::output::OutputConfig::from_global(cli.json, cli.verbose, cli.quiet, cli.no_color);

    match cli.command {
        Resource::Book { command } => handle_book(command, &output)?,
        Resource::Chapter { command } => handle_chapter(command, &output)?,
        Resource::Metadata { command } => handle_metadata(command, &output)?,
        Resource::Toc { command } => handle_toc(command, &output)?,
        Resource::Spine { command } => handle_spine(command, &output)?,
        Resource::Asset { command } => handle_asset(command, &output)?,
        Resource::Content { command } => handle_content(command, &output)?,
    }

    Ok(())
}

fn handle_book(command: cli::book::BookCommand, output: &cli::output::OutputConfig) -> Result<()> {
    use cli::book::BookCommand;

    match command {
        BookCommand::Info { file } => {
            let book = epub::reader::read_epub(&file)
                .with_context(|| format!("failed to read {}", file.display()))?;

            let total_size: usize = book.resources.values().map(|v| v.len()).sum();
            let opf_dir = book.detect_opf_dir();

            if output.json {
                let mut info = serde_json::json!({
                    "title": book.metadata.titles.first().unwrap_or(&"(untitled)".to_string()),
                    "creators": book.metadata.creators,
                    "languages": book.metadata.languages,
                    "epub_version": book.navigation.epub_version.to_string(),
                    "chapters": book.spine.len(),
                    "assets": book.manifest.len(),
                });
                if output.verbose {
                    info["opf_dir"] = serde_json::json!(if opf_dir.is_empty() {
                        "(root)"
                    } else {
                        &opf_dir
                    });
                    info["total_size_bytes"] = serde_json::json!(total_size);
                    info["identifiers"] = serde_json::json!(book.metadata.identifiers);
                    if let Some(ref cover) = book.metadata.cover_id {
                        info["cover_id"] = serde_json::json!(cover);
                    }
                }
                output.print_json(&info)?;
            } else {
                let title = book.metadata.titles.first().map_or("(untitled)", |s| s);
                println!("Title:    {title}");
                if !book.metadata.creators.is_empty() {
                    println!("Author:   {}", book.metadata.creators.join(", "));
                }
                if !book.metadata.languages.is_empty() {
                    println!("Language: {}", book.metadata.languages.join(", "));
                }
                println!("Version:  EPUB {}", book.navigation.epub_version);
                println!("Chapters: {}", book.spine.len());
                println!("Assets:   {}", book.manifest.len());
                output.detail(&format!(
                    "OPF dir:  {}",
                    if opf_dir.is_empty() {
                        "(root)"
                    } else {
                        &opf_dir
                    }
                ));
                output.detail(&format!("Size:     {}", format_size(total_size)));
                if output.verbose && !book.metadata.identifiers.is_empty() {
                    output.detail(&format!(
                        "ID:       {}",
                        book.metadata.identifiers.join("; ")
                    ));
                }
                if output.verbose
                    && let Some(ref cover) = book.metadata.cover_id
                {
                    output.detail(&format!("Cover:    {cover}"));
                }
            }
        }
        BookCommand::Extract {
            file,
            output: out_dir,
        } => {
            let book = epub::reader::read_epub(&file)
                .with_context(|| format!("failed to read {}", file.display()))?;

            let title = book
                .metadata
                .titles
                .first()
                .map(slug::slugify)
                .unwrap_or_else(|| "epub-extract".to_string());
            let output_dir = out_dir.unwrap_or_else(|| std::path::PathBuf::from(&title));

            std::fs::create_dir_all(&output_dir)?;
            extract::extract_book(&book, &output_dir)
                .with_context(|| format!("extracting to {}", output_dir.display()))?;

            output.status(&format!("Extracted to {}", output_dir.display()));
            output.detail(&format!(
                "  {} chapters, {} manifest entries",
                book.spine.len(),
                book.manifest.len()
            ));
        }
        BookCommand::Assemble {
            dir,
            output: out_file,
        } => {
            let title = dir
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| "output".to_string());
            let epub_path =
                out_file.unwrap_or_else(|| std::path::PathBuf::from(format!("{title}.epub")));

            assemble::package::package_epub(&dir, &epub_path).with_context(|| {
                format!("assembling {} to {}", dir.display(), epub_path.display())
            })?;

            output.status(&format!("Assembled {}", epub_path.display()));
            if output.verbose
                && let Ok(meta) = std::fs::metadata(&epub_path)
            {
                output.detail(&format!("  Size: {}", format_size(meta.len() as usize)));
            }
        }
        BookCommand::Validate { file } => {
            let book = epub::reader::read_epub(&file)
                .with_context(|| format!("failed to read {}", file.display()))?;

            let mut issues: Vec<String> = Vec::new();

            // Check required metadata
            if book.metadata.titles.is_empty() {
                issues.push("missing dc:title".to_string());
            }
            if book.metadata.languages.is_empty() {
                issues.push("missing dc:language".to_string());
            }
            if book.metadata.identifiers.is_empty() {
                issues.push("missing dc:identifier".to_string());
            }

            // Check spine references exist in manifest
            for spine_item in &book.spine {
                if !book.manifest.iter().any(|m| m.id == spine_item.idref) {
                    issues.push(format!(
                        "spine references missing manifest item: {}",
                        spine_item.idref
                    ));
                }
            }

            // Check empty spine
            if book.spine.is_empty() {
                issues.push("spine is empty".to_string());
            }

            if output.json {
                let json = serde_json::json!({
                    "valid": issues.is_empty(),
                    "issues": issues,
                });
                output.print_json(&json)?;
            } else if issues.is_empty() {
                println!("{}: valid", file.display());
            } else {
                println!("{}: {} issue(s)", file.display(), issues.len());
                for issue in &issues {
                    println!("  - {issue}");
                }
            }
            output.detail(&format!(
                "  Checked: metadata, spine references, {} manifest items",
                book.manifest.len()
            ));
        }
    }

    Ok(())
}

fn handle_chapter(
    command: cli::chapter::ChapterCommand,
    output: &cli::output::OutputConfig,
) -> Result<()> {
    use cli::chapter::ChapterCommand;

    match command {
        ChapterCommand::List { file } => {
            let book = epub::reader::read_epub(&file)
                .with_context(|| format!("failed to read {}", file.display()))?;
            let opf_dir = book.detect_opf_dir();

            if output.verbose {
                let rows: Vec<Vec<String>> = book
                    .spine
                    .iter()
                    .enumerate()
                    .map(|(i, item)| {
                        let manifest_item = book.manifest.iter().find(|m| m.id == item.idref);
                        let href = manifest_item.map_or("-".to_string(), |m| m.href.clone());
                        let full_path = if opf_dir.is_empty() {
                            href.clone()
                        } else {
                            format!("{opf_dir}{href}")
                        };
                        let size = book.resources.get(&full_path).map_or(0, |v| v.len());
                        vec![i.to_string(), item.idref.clone(), href, format_size(size)]
                    })
                    .collect();

                if output.json {
                    let items: Vec<_> = rows
                        .iter()
                        .map(|r| serde_json::json!({"index": r[0], "id": r[1], "href": r[2], "size": r[3]}))
                        .collect();
                    output.print_json(&items)?;
                } else {
                    output.print_table(&["#", "ID", "HREF", "SIZE"], &rows);
                }
            } else {
                let rows: Vec<Vec<String>> = book
                    .spine
                    .iter()
                    .enumerate()
                    .map(|(i, item)| {
                        let manifest_item = book.manifest.iter().find(|m| m.id == item.idref);
                        let href = manifest_item.map_or("-", |m| &m.href);
                        vec![i.to_string(), item.idref.clone(), href.to_string()]
                    })
                    .collect();

                if output.json {
                    let items: Vec<_> = rows
                        .iter()
                        .map(|r| serde_json::json!({"index": r[0], "id": r[1], "href": r[2]}))
                        .collect();
                    output.print_json(&items)?;
                } else {
                    output.print_table(&["#", "ID", "HREF"], &rows);
                }
            }
        }
        ChapterCommand::Extract {
            file,
            id,
            output: out_file,
        } => {
            let book = epub::reader::read_epub(&file)
                .with_context(|| format!("failed to read {}", file.display()))?;

            let md = extract::extract_single_chapter(&book, &id)?;

            if let Some(path) = out_file {
                std::fs::write(&path, &md)?;
                output.status(&format!("Extracted to {}", path.display()));
            } else {
                print!("{md}");
            }
        }
        ChapterCommand::Add {
            file,
            markdown,
            after,
            title,
        } => {
            let out = output;
            manipulate::meta_edit::modify_epub(&file, |book| {
                let id = manipulate::chapter_manage::add_chapter(
                    book,
                    &markdown,
                    after.as_deref(),
                    title.as_deref(),
                )?;
                out.status(&format!("Added chapter: {id}"));
                Ok(())
            })
            .with_context(|| format!("adding chapter to {}", file.display()))?;
        }
        ChapterCommand::Remove { file, id } => {
            let out = output;
            manipulate::meta_edit::modify_epub(&file, |book| {
                let removed = manipulate::chapter_manage::remove_chapter(book, &id)?;
                out.status(&format!("Removed chapter: {removed}"));
                Ok(())
            })
            .with_context(|| format!("removing chapter from {}", file.display()))?;
        }
        ChapterCommand::Reorder { file, from, to } => {
            manipulate::meta_edit::modify_epub(&file, |book| {
                manipulate::chapter_manage::reorder_chapter(book, from, to)
            })
            .with_context(|| format!("reordering chapters in {}", file.display()))?;
            output.status(&format!("Moved chapter {from} to {to}"));
        }
    }

    Ok(())
}

fn handle_metadata(
    command: cli::metadata::MetadataCommand,
    output: &cli::output::OutputConfig,
) -> Result<()> {
    use cli::metadata::MetadataCommand;

    match command {
        MetadataCommand::Show { file } => {
            let book = epub::reader::read_epub(&file)
                .with_context(|| format!("failed to read {}", file.display()))?;

            if output.json {
                output.print_json(&book.metadata)?;
            } else {
                let m = &book.metadata;
                if !m.titles.is_empty() {
                    println!("Title:       {}", m.titles.join("; "));
                }
                if !m.creators.is_empty() {
                    println!("Creator:     {}", m.creators.join("; "));
                }
                if !m.identifiers.is_empty() {
                    println!("Identifier:  {}", m.identifiers.join("; "));
                }
                if !m.languages.is_empty() {
                    println!("Language:    {}", m.languages.join("; "));
                }
                if !m.publishers.is_empty() {
                    println!("Publisher:   {}", m.publishers.join("; "));
                }
                if !m.dates.is_empty() {
                    println!("Date:        {}", m.dates.join("; "));
                }
                if let Some(ref desc) = m.description {
                    println!("Description: {desc}");
                }
                if !m.subjects.is_empty() {
                    println!("Subjects:    {}", m.subjects.join("; "));
                }
                if let Some(ref rights) = m.rights {
                    println!("Rights:      {rights}");
                }
            }
        }
        MetadataCommand::Set { file, field, value } => {
            manipulate::meta_edit::modify_epub(&file, |book| {
                manipulate::meta_edit::set_field(book, &field, &value)
            })
            .with_context(|| format!("modifying {}", file.display()))?;
            output.status(&format!("Set {field} = {value}"));
        }
        MetadataCommand::Remove { file, field } => {
            manipulate::meta_edit::modify_epub(&file, |book| {
                manipulate::meta_edit::remove_field(book, &field)
            })
            .with_context(|| format!("modifying {}", file.display()))?;
            output.status(&format!("Removed {field}"));
        }
        MetadataCommand::Import { file, metadata } => {
            manipulate::meta_edit::modify_epub(&file, |book| {
                manipulate::meta_edit::import_metadata(book, &metadata)
            })
            .with_context(|| format!("importing metadata to {}", file.display()))?;
            output.status(&format!("Imported metadata from {}", metadata.display()));
        }
        MetadataCommand::Export {
            file,
            output: out_file,
        } => {
            let book = epub::reader::read_epub(&file)
                .with_context(|| format!("failed to read {}", file.display()))?;
            let yaml_path = out_file.unwrap_or_else(|| std::path::PathBuf::from("metadata.yml"));
            manipulate::meta_edit::export_metadata(&book, &yaml_path)?;
            output.status(&format!("Exported metadata to {}", yaml_path.display()));
        }
    }

    Ok(())
}

fn handle_toc(command: cli::toc::TocCommand, output: &cli::output::OutputConfig) -> Result<()> {
    use cli::toc::TocCommand;

    match command {
        TocCommand::Show { file, depth } => {
            let book = epub::reader::read_epub(&file)
                .with_context(|| format!("failed to read {}", file.display()))?;

            if output.json {
                fn nav_to_json(points: &[epub::NavPoint]) -> Vec<serde_json::Value> {
                    points
                        .iter()
                        .map(|p| {
                            serde_json::json!({
                                "label": p.label,
                                "href": p.href,
                                "children": nav_to_json(&p.children),
                            })
                        })
                        .collect()
                }
                let json = nav_to_json(&book.navigation.toc);
                output.print_json(&json)?;
            } else {
                fn print_toc(points: &[epub::NavPoint], indent: usize, max_depth: Option<usize>) {
                    if let Some(max) = max_depth
                        && indent >= max
                    {
                        return;
                    }
                    for point in points {
                        let prefix = "  ".repeat(indent);
                        println!("{prefix}- {}", point.label);
                        print_toc(&point.children, indent + 1, max_depth);
                    }
                }
                print_toc(&book.navigation.toc, 0, depth);
            }
        }
        TocCommand::Set { file, toc } => {
            let toc_content = std::fs::read_to_string(&toc)?;
            manipulate::meta_edit::modify_epub(&file, |book| {
                manipulate::toc_edit::set_toc_from_markdown(book, &toc_content)
            })
            .with_context(|| format!("setting TOC on {}", file.display()))?;
            output.status(&format!("TOC updated from {}", toc.display()));
        }
        TocCommand::Generate { file, depth } => {
            manipulate::meta_edit::modify_epub(&file, |book| {
                manipulate::toc_edit::generate_toc(book, depth)
            })
            .with_context(|| format!("generating TOC for {}", file.display()))?;
            output.status("TOC generated from headings");
        }
    }

    Ok(())
}

fn handle_spine(
    command: cli::spine::SpineCommand,
    output: &cli::output::OutputConfig,
) -> Result<()> {
    use cli::spine::SpineCommand;

    match command {
        SpineCommand::List { file } => {
            let book = epub::reader::read_epub(&file)
                .with_context(|| format!("failed to read {}", file.display()))?;

            let rows: Vec<Vec<String>> = book
                .spine
                .iter()
                .enumerate()
                .map(|(i, item)| {
                    vec![
                        i.to_string(),
                        item.idref.clone(),
                        if item.linear { "yes" } else { "no" }.to_string(),
                    ]
                })
                .collect();

            if output.json {
                let items: Vec<_> = rows
                    .iter()
                    .map(|r| serde_json::json!({"index": r[0], "idref": r[1], "linear": r[2]}))
                    .collect();
                output.print_json(&items)?;
            } else {
                output.print_table(&["#", "IDREF", "LINEAR"], &rows);
            }
        }
        SpineCommand::Reorder { file, from, to } => {
            manipulate::meta_edit::modify_epub(&file, |book| {
                manipulate::toc_edit::reorder_spine(book, from, to)
            })
            .with_context(|| format!("reordering spine in {}", file.display()))?;
            output.status(&format!("Moved spine item {from} to {to}"));
        }
        SpineCommand::Set { file, spine } => {
            let content = std::fs::read_to_string(&spine)?;
            let idrefs: Vec<String> = serde_yaml_ng::from_str(&content)?;
            manipulate::meta_edit::modify_epub(&file, |book| {
                manipulate::toc_edit::set_spine_order(book, &idrefs)
            })
            .with_context(|| format!("setting spine on {}", file.display()))?;
            output.status(&format!("Spine order updated from {}", spine.display()));
        }
    }

    Ok(())
}

fn handle_asset(
    command: cli::asset::AssetCommand,
    output: &cli::output::OutputConfig,
) -> Result<()> {
    use cli::asset::AssetCommand;

    match command {
        AssetCommand::List { file, r#type } => {
            let book = epub::reader::read_epub(&file)
                .with_context(|| format!("failed to read {}", file.display()))?;
            let opf_dir = book.detect_opf_dir();

            let items: Vec<_> = book
                .manifest
                .iter()
                .filter(|item| {
                    if let Some(ref filter) = r#type {
                        match filter.as_str() {
                            "image" => item.media_type.starts_with("image/"),
                            "css" => item.media_type == "text/css",
                            "font" => {
                                item.media_type.contains("font")
                                    || item.media_type == "application/vnd.ms-opentype"
                            }
                            "audio" => item.media_type.starts_with("audio/"),
                            _ => true,
                        }
                    } else {
                        true
                    }
                })
                .collect();

            if output.verbose {
                let rows: Vec<Vec<String>> = items
                    .iter()
                    .map(|item| {
                        let full_path = if opf_dir.is_empty() {
                            item.href.clone()
                        } else {
                            format!("{opf_dir}{}", item.href)
                        };
                        let size = book.resources.get(&full_path).map_or(0, |v| v.len());
                        vec![
                            item.id.clone(),
                            item.href.clone(),
                            item.media_type.clone(),
                            format_size(size),
                        ]
                    })
                    .collect();

                if output.json {
                    let json: Vec<_> = items
                        .iter()
                        .map(|item| {
                            let full_path = if opf_dir.is_empty() {
                                item.href.clone()
                            } else {
                                format!("{opf_dir}{}", item.href)
                            };
                            let size = book.resources.get(&full_path).map_or(0, |v| v.len());
                            serde_json::json!({
                                "id": item.id,
                                "href": item.href,
                                "media_type": item.media_type,
                                "size_bytes": size,
                            })
                        })
                        .collect();
                    output.print_json(&json)?;
                } else {
                    output.print_table(&["ID", "HREF", "MEDIA-TYPE", "SIZE"], &rows);
                }
            } else {
                let rows: Vec<Vec<String>> = items
                    .iter()
                    .map(|item| vec![item.id.clone(), item.href.clone(), item.media_type.clone()])
                    .collect();

                if output.json {
                    let json: Vec<_> = items
                        .iter()
                        .map(|item| {
                            serde_json::json!({
                                "id": item.id,
                                "href": item.href,
                                "media_type": item.media_type,
                            })
                        })
                        .collect();
                    output.print_json(&json)?;
                } else {
                    output.print_table(&["ID", "HREF", "MEDIA-TYPE"], &rows);
                }
            }
        }
        AssetCommand::Extract {
            file,
            asset_path,
            output: out_file,
        } => {
            let book = epub::reader::read_epub(&file)
                .with_context(|| format!("failed to read {}", file.display()))?;

            let data = book
                .resources
                .iter()
                .find(|(k, _)| k.ends_with(&asset_path) || **k == asset_path)
                .map(|(_, v)| v)
                .ok_or_else(|| anyhow::anyhow!("asset not found: {asset_path}"))?;

            if let Some(path) = out_file {
                std::fs::write(&path, data)?;
                output.status(&format!("Extracted to {}", path.display()));
                output.detail(&format!("  Size: {}", format_size(data.len())));
            } else {
                use std::io::Write;
                std::io::stdout().write_all(data)?;
            }
        }
        AssetCommand::ExtractAll {
            file,
            output: out_dir,
        } => {
            let book = epub::reader::read_epub(&file)
                .with_context(|| format!("failed to read {}", file.display()))?;

            let output_dir = out_dir.unwrap_or_else(|| std::path::PathBuf::from("assets"));
            let opf_dir = extract::asset_extract::build_path_map(&book, "");
            let _ = opf_dir; // path_map not needed here
            extract::asset_extract::extract_assets(&book, &output_dir, "")?;
            output.status(&format!("Assets extracted to {}", output_dir.display()));
        }
        AssetCommand::Add {
            file,
            asset,
            media_type,
        } => {
            let out = output;
            manipulate::meta_edit::modify_epub(&file, |book| {
                let id = manipulate::asset_manage::add_asset(book, &asset, media_type.as_deref())?;
                out.status(&format!("Added asset: {id}"));
                Ok(())
            })
            .with_context(|| format!("adding asset to {}", file.display()))?;
        }
        AssetCommand::Remove { file, asset_path } => {
            manipulate::meta_edit::modify_epub(&file, |book| {
                manipulate::asset_manage::remove_asset(book, &asset_path)
            })
            .with_context(|| format!("removing asset from {}", file.display()))?;
            output.status(&format!("Removed asset: {asset_path}"));
        }
    }

    Ok(())
}

fn handle_content(
    command: cli::content::ContentCommand,
    output: &cli::output::OutputConfig,
) -> Result<()> {
    use cli::content::ContentCommand;
    match command {
        ContentCommand::Search {
            file,
            pattern,
            chapter,
            regex: use_regex,
        } => {
            let book = epub::reader::read_epub(&file)
                .with_context(|| format!("failed to read {}", file.display()))?;

            let matches =
                manipulate::content_edit::search(&book, &pattern, chapter.as_deref(), use_regex)?;

            if output.json {
                let json: Vec<_> = matches
                    .iter()
                    .map(|m| {
                        serde_json::json!({
                            "chapter_id": m.chapter_id,
                            "chapter_href": m.chapter_href,
                            "line": m.line_number,
                            "context": m.context,
                        })
                    })
                    .collect();
                output.print_json(&json)?;
            } else {
                for m in &matches {
                    println!("{}:{}: {}", m.chapter_href, m.line_number, m.context);
                }
                output.status(&format!("\n{} match(es) found", matches.len()));
            }
        }
        ContentCommand::Replace {
            file,
            pattern,
            replacement,
            chapter,
            regex: use_regex,
            dry_run,
        } => {
            if dry_run {
                let book = epub::reader::read_epub(&file)
                    .with_context(|| format!("failed to read {}", file.display()))?;
                let matches = manipulate::content_edit::search(
                    &book,
                    &pattern,
                    chapter.as_deref(),
                    use_regex,
                )?;
                println!("Dry run: {} match(es) would be replaced", matches.len());
                for m in &matches {
                    println!("  {}:{}: {}", m.chapter_href, m.line_number, m.context);
                }
            } else {
                let mut count = 0;
                manipulate::meta_edit::modify_epub(&file, |book| {
                    count = manipulate::content_edit::replace(
                        book,
                        &pattern,
                        &replacement,
                        chapter.as_deref(),
                        use_regex,
                    )?;
                    Ok(())
                })
                .with_context(|| format!("replacing in {}", file.display()))?;
                output.status(&format!("Replaced {count} occurrence(s)"));
            }
        }
        ContentCommand::Headings { file, restructure } => {
            if let Some(mapping) = restructure {
                let mut count = 0;
                manipulate::meta_edit::modify_epub(&file, |book| {
                    count = manipulate::content_edit::restructure_headings(book, &mapping)?;
                    Ok(())
                })
                .with_context(|| format!("restructuring headings in {}", file.display()))?;
                output.status(&format!("Restructured {count} heading(s)"));
            } else {
                let book = epub::reader::read_epub(&file)
                    .with_context(|| format!("failed to read {}", file.display()))?;
                let headings = manipulate::content_edit::list_headings(&book)?;
                if output.json {
                    let json: Vec<_> = headings
                        .iter()
                        .map(|(href, level, text)| {
                            serde_json::json!({
                                "href": href,
                                "level": level,
                                "text": text,
                            })
                        })
                        .collect();
                    output.print_json(&json)?;
                } else {
                    for (href, level, text) in &headings {
                        let indent = "  ".repeat(*level - 1);
                        println!("{indent}h{level}: {text}  ({href})");
                    }
                }
            }
        }
    }

    Ok(())
}
