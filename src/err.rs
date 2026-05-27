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

use std::io;
use std::path::PathBuf;

use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
    #[error("config file not found; tried {0}")]
    NoCfg(Paths),
    #[error("failed to read config {path}: {source}")]
    Read { path: PathBuf, source: io::Error },
    #[error("failed to parse config {path}: {source}")]
    Parse { path: PathBuf, source: toml::de::Error },
    #[error("missing file name in path {0}")]
    Name(PathBuf),
    #[error("missing file name in remote path {0}")]
    Remote(String),
    #[error("ftp error: {0}")]
    Ftp(#[from] suppaftp::FtpError),
    #[error("tls error: {0}")]
    Tls(String),
    #[error("io error: {0}")]
    Io(#[from] io::Error),
}

#[derive(Debug)]
pub struct Paths(pub Vec<PathBuf>);

impl std::fmt::Display for Paths {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let paths = self.0.iter().map(|path| path.display().to_string()).collect::<Vec<_>>().join(", ");
        f.write_str(&paths)
    }
}
