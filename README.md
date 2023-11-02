# cor-args
A Rust library providing [Chain of Responsibility](https://en.wikipedia.org/wiki/Chain-of-responsibility_pattern) command line argument parsing.

# Example

The following example will assign `config_path` according to the following:

1. Look for a command-line argument named `--config`
1. Look for an environment variable named `MYAPP_config`
1. Default to `~/.config/myapp/default.yaml`

```rust
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
