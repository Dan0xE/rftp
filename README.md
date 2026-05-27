# rftp

Rust CLI for common FTP operations with a TOML config file.

`rftp` uses [`suppaftp`](https://crates.io/crates/suppaftp) instead of the older `ftp` crate (the crate I initially started this project with a while ago) because `suppaftp` is maintained! :)

## Config

By default, `rftp` looks for:

1. `ftp.toml`
2. `~/.config/rftp/config.toml`

Pass a specific config with `-c`:

```sh
rftp -c ./ftp.toml ls
```

See [`ftp.toml.example`](ftp.toml.example).

## Usage

```sh
rftp ls
rftp ls uploads
rftp get /public_html/index.html
rftp get /public_html/index.html ./index.html
rftp put ./index.html
rftp put ./index.html index.html
rftp rm old.html
rftp mkdir assets
rftp rmdir empty-dir
rftp pwd
rftp cd /public_html
```
