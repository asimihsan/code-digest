/*
 * Copyright (c) 2023 Asim Ihsan.
 *
 * This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/.
 *
 * SPDX-License-Identifier: MPL-2.0
 */

use std::path::PathBuf;

use clap::Parser;

#[derive(thiserror::Error, Debug)]
pub enum ConfigError {
    #[error("Failed to parse CLI arguments: {0}")]
    CliError(#[from] clap::Error),

    #[error("Display help or version")]
    DisplayHelpOrVersion(clap::Error),
}

#[derive(Debug, Clone)]
pub struct AppConfig {
    /// The path to the directory containing the files.
    pub directory: String,

    /// Additional directories to ignore (optional, zero or more)
    pub ignore: Vec<PathBuf>,

    /// Glob patterns for which to include the full file contents, e.g. `*.md` (optional, zero or more)
    pub include: Vec<String>,

    /// Print a file tree for each directory (optional, default false)
    pub tree: bool,
}

impl AppConfig {
    pub fn new(args: &[String]) -> Result<Self, ConfigError> {
        let cli = match Cli::try_parse_from(args) {
            Ok(cli) => cli,
            Err(e)
                if e.kind() == clap::error::ErrorKind::DisplayHelp
                    || e.kind() == clap::error::ErrorKind::DisplayVersion =>
            {
                return Err(ConfigError::DisplayHelpOrVersion(e));
            }
            Err(e) => {
                return Err(ConfigError::CliError(e));
            }
        };
        Ok(Self {
            directory: cli.directory,
            ignore: cli.ignore,
            include: cli.include,
            tree: cli.tree,
        })
    }
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
pub struct Cli {
    /// The path to the directory containing the files.
    pub directory: String,

    /// Additional directories to ignore (optional, zero or more)
    #[clap(short = 'i', long)]
    pub ignore: Vec<PathBuf>,

    /// Glob patterns for which to include the full file contents, e.g. `*.md` (optional, zero or more)
    #[clap(short = 'I', long)]
    pub include: Vec<String>,

    /// Print a file tree for each directory (optional, default false)
    #[clap(short = 't', long)]
    pub tree: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cli_args() {
        let args = vec![
            "code-digest",
            "--ignore",
            "/path/to/ignore",
            "--include",
            "*.md",
            "--tree",
            "/path/to/directory",
        ];
        let args = args.iter().map(|s| s.to_string()).collect::<Vec<String>>();
        let app_config = AppConfig::new(&args);
        if let Err(e) = &app_config {
            panic!("Error parsing CLI arguments: {}", e);
        }
        let app_config = app_config.unwrap();

        assert_eq!(app_config.directory, "/path/to/directory");
        assert_eq!(app_config.ignore, vec![PathBuf::from("/path/to/ignore")]);
        assert_eq!(app_config.include, vec!["*.md"]);
        assert!(app_config.tree);
    }
}
