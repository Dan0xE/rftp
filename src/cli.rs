/*
Copyright (c) 2026 Daniel Oppermann

Permission is hereby granted, free of charge, to any person obtaining a copy of
this software and associated documentation files (the "Software"), to deal in
the Software without restriction, including without limitation the rights to
use, copy, modify, merge, publish, distribute, sublicense, and/or sell copies of
the Software, and to permit persons to whom the Software is furnished to do so,
subject to the following conditions:

The above copyright notice and this permission notice shall be included in all
copies or substantial portions of the Software.

THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS
FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR
COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER
IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN
CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
*/

use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(name = "rftp", version, about = "FTP command-line client")]
pub struct Cli {
    #[arg(short, long = "config", alias = "cfg", value_name = "FILE")]
    cfg: Option<PathBuf>,
    #[command(subcommand)]
    cmd: Cmd,
}

impl Cli {
    #[inline]
    pub fn cfg(&self) -> Option<&Path> {
        self.cfg.as_deref()
    }

    #[inline]
    pub fn cmd(&self) -> &Cmd {
        &self.cmd
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Subcommand)]
pub enum Cmd {
    /// List a remote directory.
    Ls { path: Option<String> },
    /// Download a remote file.
    Get { remote: String, local: Option<PathBuf> },
    /// Upload a local file.
    Put { local: PathBuf, remote: Option<String> },
    /// Remove a remote file.
    Rm { remote: String },
    /// Create a remote directory.
    Mkdir { remote: String },
    /// Remove an empty remote directory.
    Rmdir { remote: String },
    /// Print the remote working directory.
    Pwd,
    /// Change remote directory for this command and print it.
    Cd { remote: String },
}
