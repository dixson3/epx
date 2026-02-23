use std::io::{self, IsTerminal, Write};

pub struct OutputConfig {
    pub json: bool,
    pub verbose: bool,
    pub quiet: bool,
    pub no_color: bool,
}

impl OutputConfig {
    pub fn from_global(json: bool, verbose: bool, quiet: bool, no_color: bool) -> Self {
        let no_color = no_color || std::env::var("NO_COLOR").is_ok() || !io::stdout().is_terminal();
        Self {
            json,
            verbose,
            quiet,
            no_color,
        }
    }

    pub fn is_tty(&self) -> bool {
        io::stdout().is_terminal()
    }

    /// Print a status/confirmation message (suppressed in quiet mode).
    pub fn status(&self, msg: &str) {
        if !self.quiet {
            println!("{msg}");
        }
    }

    /// Print extra detail (only shown in verbose mode, suppressed in quiet mode).
    pub fn detail(&self, msg: &str) {
        if self.verbose && !self.quiet {
            println!("{msg}");
        }
    }

    pub fn print_json<T: serde::Serialize>(&self, value: &T) -> anyhow::Result<()> {
        let json = serde_json::to_string_pretty(value)?;
        writeln!(io::stdout(), "{json}")?;
        Ok(())
    }

    pub fn print_table(&self, headers: &[&str], rows: &[Vec<String>]) {
        if rows.is_empty() {
            return;
        }

        let mut widths: Vec<usize> = headers.iter().map(|h| h.len()).collect();
        for row in rows {
            for (i, cell) in row.iter().enumerate() {
                if i < widths.len() {
                    widths[i] = widths[i].max(cell.len());
                }
            }
        }

        if self.is_tty() && !self.no_color {
            let header_line: String = headers
                .iter()
                .zip(&widths)
                .map(|(h, w)| format!("{h:<w$}"))
                .collect::<Vec<_>>()
                .join("  ");
            println!("{header_line}");
            let sep: String = widths
                .iter()
                .map(|w| "-".repeat(*w))
                .collect::<Vec<_>>()
                .join("  ");
            println!("{sep}");
        } else {
            println!("{}", headers.join("\t"));
        }

        for row in rows {
            if self.is_tty() && !self.no_color {
                let line: String = row
                    .iter()
                    .zip(&widths)
                    .map(|(c, w)| format!("{c:<w$}"))
                    .collect::<Vec<_>>()
                    .join("  ");
                println!("{line}");
            } else {
                println!("{}", row.join("\t"));
            }
        }
    }
}
