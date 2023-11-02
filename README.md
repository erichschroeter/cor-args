# cor-args
A Rust library providing [Chain of Responsibility](https://en.wikipedia.org/wiki/Chain-of-responsibility_pattern) command line argument parsing.

# Examples

## Example 1
The following example will assign `config_path` according to the following sequence:

1. Look for a command-line argument named `--config`
1. Look for an environment variable named `MYAPP_config`
1. Default to `~/.config/myapp/default.yaml`

```rust
// Don't forget to add `clap` to your `Cargo.toml`
let args = clap::Command::new("myapp")
    .arg(clap::Arg::new("config").long("config"))
    .get_matches();

let config_path = ArgHandler::new(&args)
    .next(
        EnvHandler::new()
        .prefix("MYAPP_")
        .next(
            DefaultHandler::new("~/.config/myapp/default.yaml").into()
        ).into()
    );
    .handle_request("config");
let config_path = config_path.expect("No config path");
```

## Example 2
The following example will assign `some_value` according to the following sequence:

1. Look for a command-line argument named `--some_key`
1. Look for an environment variable named `MYAPP_some_key`
1. The contents of the file `/path/to/file.txt`
1. Look for a key within `file.json` named `some_key`
1. Default to `"some_value"`

```rust
// Don't forget to add `clap` to your `Cargo.toml`
let args = clap::Command::new("myapp")
    .arg(clap::Arg::new("some_key").long("some_key"))
    .get_matches();

let handler = ArgHandler::new(&args)
    .next(EnvHandler::new()
        .next(FileHandler::new("/path/to/file.txt")
            .next(JSONFileHandler::new("file.json")
                .next(DefaultHandler::new("some_value").into())
            .into())
        .into())
    .into());
let some_value = handler.handle_request("some_key");
```
