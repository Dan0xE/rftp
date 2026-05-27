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

use std::fs::File;
use std::io::{self, Read};
use std::path::Path;
use std::sync::Arc;

use suppaftp::rustls::{ClientConfig, RootCertStore, version};
use suppaftp::types::{FileType, FtpError};
use suppaftp::{FtpStream, Mode, RustlsConnector, RustlsFtpStream, Status};

use crate::Result;
use crate::cfg::{Cfg, Tls, TlsVer};

enum Conn {
    Plain(FtpStream),
    Tls(RustlsFtpStream),
}

pub struct Ftp {
    conn: Conn,
}

impl Ftp {
    pub fn open(cfg: &Cfg) -> Result<Self> {
        let conn = match cfg.tls {
            Tls::None => Conn::Plain(FtpStream::connect(cfg.addr())?),
            Tls::Explicit => {
                let stream = RustlsFtpStream::connect(cfg.addr())?;
                Conn::Tls(stream.into_secure(Self::tls(cfg)?, &cfg.host)?)
            }
            Tls::Implicit => {
                Conn::Tls(RustlsFtpStream::connect_secure_implicit(cfg.addr(), Self::tls(cfg)?, &cfg.host)?)
            }
        };

        let mut ftp = Self { conn };
        ftp.init(cfg)?;
        Ok(ftp)
    }

    fn tls(cfg: &Cfg) -> Result<RustlsConnector> {
        let roots = RootCertStore::from_iter(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
        let builder = match cfg.tls_version {
            Some(TlsVer::V12) => ClientConfig::builder_with_protocol_versions(&[&version::TLS12]),
            Some(TlsVer::V13) => ClientConfig::builder_with_protocol_versions(&[&version::TLS13]),
            None => ClientConfig::builder(),
        };
        let cfg = builder.with_root_certificates(roots).with_no_client_auth();
        Ok(RustlsConnector::from(Arc::new(cfg)))
    }

    pub fn ls(&mut self, path: Option<&str>) -> Result<Vec<String>> {
        self.with(|conn| conn.list(path)).map_err(Into::into)
    }

    pub fn get(&mut self, remote: &str, local: &Path) -> Result<u64> {
        let mut out = File::create(local)?;
        let mut read = |reader: &mut dyn Read| io::copy(reader, &mut out).map_err(FtpError::ConnectionError);
        self.with(|conn| conn.retr(remote, &mut read)).map_err(Into::into)
    }

    pub fn put(&mut self, local: &Path, remote: &str) -> Result<u64> {
        let mut file = File::open(local)?;
        self.with(|conn| conn.put_file(remote, &mut file)).map_err(Into::into)
    }

    pub fn rm(&mut self, remote: &str) -> Result<()> {
        self.with(|conn| conn.rm(remote)).map_err(Into::into)
    }

    pub fn mkdir(&mut self, remote: &str) -> Result<()> {
        self.with(|conn| conn.mkdir(remote)).map_err(Into::into)
    }

    pub fn rmdir(&mut self, remote: &str) -> Result<()> {
        self.with(|conn| conn.rmdir(remote)).map_err(Into::into)
    }

    pub fn pwd(&mut self) -> Result<String> {
        self.with(|conn| conn.pwd()).map_err(Into::into)
    }

    pub fn cd(&mut self, remote: &str) -> Result<()> {
        self.with(|conn| conn.cwd(remote)).map_err(Into::into)
    }

    pub fn quit(&mut self) -> Result<()> {
        self.with(|conn| conn.quit()).map_err(Into::into)
    }

    fn init(&mut self, cfg: &Cfg) -> Result<()> {
        if !cfg.passive {
            self.set_mode(Mode::Active);
        }

        self.with(|conn| conn.login(&cfg.user, &cfg.pass))?;
        if cfg.tls == Tls::Implicit {
            // RFC 4217 section 9 makes the data-channel protection explicit:
            // PBSZ prepares the protection buffer, and PROT P asks for a
            // private/TLS data connection. Explicit FTPS gets this from
            // suppaftp::into_secure; implicit FTPS does not, so we send it
            // here.
            // For example: test.rebex.net doesn't like if we don't do this,
            // LIST failed as a low-level TLS record overflow instead of the
            // server's real data-channel policy error (that was happening before we switched to rustls).
            self.protect_data()?;
        }
        self.with(|conn| conn.transfer_type(FileType::Binary))?;

        if let Some(dir) = cfg.remote_dir.as_deref() {
            self.cd(dir)?;
        }

        Ok(())
    }

    #[inline]
    fn set_mode(&mut self, mode: Mode) {
        match &mut self.conn {
            Conn::Plain(conn) => conn.set_mode(mode),
            Conn::Tls(conn) => conn.set_mode(mode),
        }
    }

    fn protect_data(&mut self) -> Result<()> {
        self.with(|conn| conn.custom_command("PBSZ 0", &[Status::CommandOk]))?;
        self.with(|conn| conn.custom_command("PROT P", &[Status::CommandOk]))?;
        Ok(())
    }

    #[inline]
    fn with<T>(&mut self, f: impl FnOnce(&mut dyn Wire) -> suppaftp::FtpResult<T>) -> suppaftp::FtpResult<T> {
        match &mut self.conn {
            Conn::Plain(conn) => f(conn),
            Conn::Tls(conn) => f(conn),
        }
    }
}

trait Wire {
    fn login(&mut self, user: &str, pass: &str) -> suppaftp::FtpResult<()>;
    fn transfer_type(&mut self, ty: FileType) -> suppaftp::FtpResult<()>;
    fn list(&mut self, path: Option<&str>) -> suppaftp::FtpResult<Vec<String>>;
    fn retr(
        &mut self,
        path: &str,
        read: &mut dyn FnMut(&mut dyn Read) -> suppaftp::FtpResult<u64>,
    ) -> suppaftp::FtpResult<u64>;
    fn put_file(&mut self, path: &str, file: &mut File) -> suppaftp::FtpResult<u64>;
    fn rm(&mut self, path: &str) -> suppaftp::FtpResult<()>;
    fn mkdir(&mut self, path: &str) -> suppaftp::FtpResult<()>;
    fn rmdir(&mut self, path: &str) -> suppaftp::FtpResult<()>;
    fn pwd(&mut self) -> suppaftp::FtpResult<String>;
    fn cwd(&mut self, path: &str) -> suppaftp::FtpResult<()>;
    fn quit(&mut self) -> suppaftp::FtpResult<()>;
    fn custom_command(&mut self, command: &str, expected: &[Status]) -> suppaftp::FtpResult<()>;
}

impl<T> Wire for suppaftp::ImplFtpStream<T>
where
    T: suppaftp::TlsStream,
{
    fn login(&mut self, user: &str, pass: &str) -> suppaftp::FtpResult<()> {
        self.login(user, pass)
    }

    fn transfer_type(&mut self, ty: FileType) -> suppaftp::FtpResult<()> {
        self.transfer_type(ty)
    }

    fn list(&mut self, path: Option<&str>) -> suppaftp::FtpResult<Vec<String>> {
        self.list(path)
    }

    fn retr(
        &mut self,
        path: &str,
        read: &mut dyn FnMut(&mut dyn Read) -> suppaftp::FtpResult<u64>,
    ) -> suppaftp::FtpResult<u64> {
        self.retr(path, read)
    }

    fn put_file(&mut self, path: &str, file: &mut File) -> suppaftp::FtpResult<u64> {
        self.put_file(path, file)
    }

    fn rm(&mut self, path: &str) -> suppaftp::FtpResult<()> {
        self.rm(path)
    }

    fn mkdir(&mut self, path: &str) -> suppaftp::FtpResult<()> {
        self.mkdir(path)
    }

    fn rmdir(&mut self, path: &str) -> suppaftp::FtpResult<()> {
        self.rmdir(path)
    }

    fn pwd(&mut self) -> suppaftp::FtpResult<String> {
        self.pwd()
    }

    fn cwd(&mut self, path: &str) -> suppaftp::FtpResult<()> {
        self.cwd(path)
    }

    fn quit(&mut self) -> suppaftp::FtpResult<()> {
        self.quit()
    }

    fn custom_command(&mut self, command: &str, expected: &[Status]) -> suppaftp::FtpResult<()> {
        self.custom_command(command, expected).map(|_| ())
    }
}
