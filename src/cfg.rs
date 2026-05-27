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
use std::{env, fs};

use serde::{Deserialize, Deserializer};

use crate::Error;
use crate::err::{Paths, Result};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Cfg {
    pub host: String,
    pub port: u16,
    pub user: String,
    pub pass: String,
    pub tls: Tls,
    pub tls_version: Option<TlsVer>,
    pub passive: bool,
    pub remote_dir: Option<String>,
    pub local_dir: Option<PathBuf>,
}

#[derive(Clone, Copy, Debug, Default, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Tls {
    #[default]
    None,
    Explicit,
    Implicit,
}

#[derive(Clone, Copy, Debug, Deserialize, PartialEq, Eq)]
pub enum TlsVer {
    #[serde(rename = "1.2", alias = "tls1.2", alias = "TLS1.2")]
    V12,
    #[serde(rename = "1.3", alias = "tls1.3", alias = "TLS1.3")]
    V13,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct Raw {
    host: String,
    port: Option<u16>,
    user: String,
    pass: String,
    #[serde(default)]
    tls: Tls,
    #[serde(default)]
    tls_version: Option<TlsVer>,
    #[serde(default = "Cfg::yes")]
    passive: bool,
    #[serde(default)]
    remote_dir: Option<String>,
    #[serde(default)]
    local_dir: Option<PathBuf>,
}

impl Tls {
    const fn port(self) -> u16 {
        match self {
            Self::None | Self::Explicit => 21,
            Self::Implicit => 990,
        }
    }
}

impl<'de> Deserialize<'de> for Cfg {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        Ok(Raw::deserialize(deserializer)?.into())
    }
}

impl From<Raw> for Cfg {
    fn from(raw: Raw) -> Self {
        Self {
            host: raw.host,
            port: raw.port.unwrap_or_else(|| raw.tls.port()),
            user: raw.user,
            pass: raw.pass,
            tls: raw.tls,
            tls_version: raw.tls_version,
            passive: raw.passive,
            remote_dir: raw.remote_dir.map(|dir| dir.trim().to_owned()).filter(|dir| !dir.is_empty()),
            local_dir: raw.local_dir,
        }
    }
}

impl Cfg {
    pub fn find(path: Option<&Path>) -> Result<Self> {
        match path {
            Some(path) => Self::load(path),
            None => {
                let paths = Self::paths();
                if let Some(path) = Self::pick(&paths) { Self::load(path) } else { Err(Error::NoCfg(Paths(paths))) }
            }
        }
    }

    pub fn load(path: impl AsRef<Path>) -> Result<Self> {
        let path = Self::home(path.as_ref());
        let text = fs::read_to_string(&path).map_err(|source| Error::Read { path: path.clone(), source })?;
        toml::from_str(&text).map_err(|source| Error::Parse { path, source })
    }

    pub fn paths() -> Vec<PathBuf> {
        let mut paths = vec![PathBuf::from("ftp.toml")];
        if let Some(home) = env::var_os("HOME") {
            paths.push(PathBuf::from(home).join(".config/rftp/config.toml"));
        }
        paths
    }

    pub fn pick(paths: &[PathBuf]) -> Option<&Path> {
        paths.iter().find(|path| path.is_file()).map(PathBuf::as_path)
    }

    pub fn addr(&self) -> String {
        format!("{}:{}", self.host, self.port)
    }

    pub fn local(&self, path: impl AsRef<Path>) -> PathBuf {
        let path = path.as_ref();
        if path.is_absolute() {
            return path.to_path_buf();
        }

        self.local_dir.as_ref().map(|dir| Self::home(dir).join(path)).unwrap_or_else(|| path.to_path_buf())
    }

    pub fn get_path(&self, remote: &str, local: Option<&Path>) -> Result<PathBuf> {
        let local = match local {
            Some(path) => path.to_path_buf(),
            None => PathBuf::from(Self::remote_name(remote)?),
        };
        Ok(self.local(local))
    }

    pub fn put_path(&self, local: &Path, remote: Option<&str>) -> Result<(PathBuf, String)> {
        let local_path = self.local(local);
        let remote_path = remote.map(ToOwned::to_owned).map(Ok).unwrap_or_else(|| Self::file_name(local))?;
        Ok((local_path, remote_path))
    }

    fn home(path: &Path) -> PathBuf {
        let text = path.to_string_lossy();
        if text == "~" {
            return env::var_os("HOME").map(PathBuf::from).unwrap_or_else(|| path.into());
        }
        if let Some(rest) = text.strip_prefix("~/") {
            return env::var_os("HOME").map(|home| PathBuf::from(home).join(rest)).unwrap_or_else(|| path.into());
        }
        path.to_path_buf()
    }

    fn file_name(path: &Path) -> Result<String> {
        path.file_name()
            .and_then(|name| name.to_str())
            .map(ToOwned::to_owned)
            .ok_or_else(|| Error::Name(path.to_path_buf()))
    }

    fn remote_name(path: &str) -> Result<String> {
        path.trim_end_matches('/')
            .rsplit('/')
            .find(|part| !part.is_empty())
            .map(ToOwned::to_owned)
            .ok_or_else(|| Error::Remote(path.to_owned()))
    }

    #[inline]
    const fn yes() -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads_defaults() {
        let file = tempfile::NamedTempFile::new().unwrap();
        fs::write(
            file.path(),
            r#"
host = "ftp.example.com"
user = "me"
pass = "secret"
"#,
        )
        .unwrap();

        let cfg = Cfg::load(file.path()).unwrap();

        assert_eq!(cfg.host, "ftp.example.com");
        assert_eq!(cfg.port, 21);
        assert!(cfg.passive);
        assert_eq!(cfg.tls, Tls::None);
        assert_eq!(cfg.tls_version, None);
    }

    #[test]
    fn loads_tls_modes() {
        let explicit = toml::from_str::<Cfg>(
            r#"
host = "ftp.example.com"
user = "me"
pass = "secret"
tls = "explicit"
"#,
        )
        .unwrap();
        let implicit = toml::from_str::<Cfg>(
            r#"
host = "ftp.example.com"
user = "me"
pass = "secret"
tls = "implicit"
"#,
        )
        .unwrap();

        assert_eq!(explicit.tls, Tls::Explicit);
        assert_eq!(explicit.port, 21);
        assert_eq!(implicit.tls, Tls::Implicit);
        assert_eq!(implicit.port, 990);
    }

    #[test]
    fn loads_tls_version() {
        let cfg = toml::from_str::<Cfg>(
            r#"
host = "ftp.example.com"
user = "me"
pass = "secret"
tls = "implicit"
tls_version = "1.2"
"#,
        )
        .unwrap();

        assert_eq!(cfg.tls_version, Some(TlsVer::V12));
    }

    #[test]
    fn picks_first_existing_path() {
        let dir = tempfile::tempdir().unwrap();
        let missing = dir.path().join("missing.toml");
        let first = dir.path().join("ftp.toml");
        let second = dir.path().join("config.toml");
        fs::write(&first, "").unwrap();
        fs::write(&second, "").unwrap();

        let paths = vec![missing, first.clone(), second];

        assert_eq!(Cfg::pick(&paths), Some(first.as_path()));
    }
}
