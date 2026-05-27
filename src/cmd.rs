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

use std::path::PathBuf;

use crate::Result;
use crate::cfg::Cfg;
use crate::cli::Cmd;
use crate::ftp::Ftp;

pub trait Run {
    fn run(&self, ftp: &mut Ftp, cfg: &Cfg) -> Result<()>;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Act {
    Ls { path: Option<String> },
    Get { remote: String, local: PathBuf },
    Put { local: PathBuf, remote: String },
    Rm { remote: String },
    Mkdir { remote: String },
    Rmdir { remote: String },
    Pwd,
    Cd { remote: String },
}

impl Cmd {
    pub fn plan(&self, cfg: &Cfg) -> Result<Act> {
        match self {
            Self::Ls { path } => Ok(Act::Ls { path: path.clone() }),
            Self::Get { remote, local } => {
                Ok(Act::Get { remote: remote.clone(), local: cfg.get_path(remote, local.as_deref())? })
            }
            Self::Put { local, remote } => {
                let (local, remote) = cfg.put_path(local, remote.as_deref())?;
                Ok(Act::Put { local, remote })
            }
            Self::Rm { remote } => Ok(Act::Rm { remote: remote.clone() }),
            Self::Mkdir { remote } => Ok(Act::Mkdir { remote: remote.clone() }),
            Self::Rmdir { remote } => Ok(Act::Rmdir { remote: remote.clone() }),
            Self::Pwd => Ok(Act::Pwd),
            Self::Cd { remote } => Ok(Act::Cd { remote: remote.clone() }),
        }
    }
}

impl Run for Cmd {
    fn run(&self, ftp: &mut Ftp, cfg: &Cfg) -> Result<()> {
        match self.plan(cfg)? {
            Act::Ls { path } => {
                for line in ftp.ls(path.as_deref())? {
                    println!("{line}");
                }
            }
            Act::Get { remote, local } => {
                let bytes = ftp.get(&remote, &local)?;
                println!("got {remote} -> {} ({bytes} bytes)", local.display());
            }
            Act::Put { local, remote } => {
                let bytes = ftp.put(&local, &remote)?;
                println!("put {} -> {remote} ({bytes} bytes)", local.display());
            }
            Act::Rm { remote } => {
                ftp.rm(&remote)?;
                println!("removed {remote}");
            }
            Act::Mkdir { remote } => {
                ftp.mkdir(&remote)?;
                println!("created {remote}");
            }
            Act::Rmdir { remote } => {
                ftp.rmdir(&remote)?;
                println!("removed {remote}");
            }
            Act::Pwd => {
                println!("{}", ftp.pwd()?);
            }
            Act::Cd { remote } => {
                ftp.cd(&remote)?;
                println!("{}", ftp.pwd()?);
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;
    use crate::cfg::Tls;

    fn cfg() -> Cfg {
        Cfg {
            host: "ftp.example.com".into(),
            port: 21,
            user: "me".into(),
            pass: "secret".into(),
            tls: Tls::None,
            tls_version: None,
            passive: true,
            remote_dir: Some("/public_html".into()),
            local_dir: Some(PathBuf::from("site")),
        }
    }

    #[test]
    fn resolves_get_default_local() {
        let act = Cmd::Get { remote: "/assets/app.css".into(), local: None }.plan(&cfg()).unwrap();

        assert_eq!(act, Act::Get { remote: "/assets/app.css".into(), local: PathBuf::from("site/app.css") });
    }

    #[test]
    fn resolves_put_default_remote() {
        let act = Cmd::Put { local: PathBuf::from("app.css"), remote: None }.plan(&cfg()).unwrap();

        assert_eq!(act, Act::Put { local: PathBuf::from("site/app.css"), remote: "app.css".into() });
    }

    #[test]
    fn keeps_absolute_local_path() {
        let act =
            Cmd::Get { remote: "app.css".into(), local: Some(PathBuf::from("/tmp/app.css")) }.plan(&cfg()).unwrap();

        assert_eq!(act, Act::Get { remote: "app.css".into(), local: PathBuf::from("/tmp/app.css") });
    }
}
