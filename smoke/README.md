# Smoke Configs

Public FTP/FTPS endpoints for manual smoke tests.
The servers below have been pulled from: https://www.sftp.net/public-online-ftp-servers :)

Examples:

```sh
cargo run -- -c smoke/rebex.toml ls
cargo run -- -c smoke/rebex-explicit.toml get readme.txt
cargo run -- -c smoke/rebex-implicit.toml ls
cargo run -- -c smoke/wftp-implicit.toml ls download
```

Downloads go to `target/smoke`.
