#[cfg(feature = "config")]
use cor_args::{ConfigHandler, DefaultHandler, EnvHandler, FileHandler, Handler};

/// This example can be run multiple ways to test out the Chain of Responsibility.
///
/// # Testing the ArgHandler
/// Create a YAML file `default.yml` in the current directory.
///
/// ```yaml
/// ---
/// verbosity: "warn"
/// ```
///
/// ```bash
/// cargo run --features config --example config
/// ```
///
/// # Testing the EnvHandler
/// ```bash
/// verbosity=info cargo run --features config --example config
/// ```
///
/// # Testing the FileHandler
/// ```bash
/// echo debug > verbosity.txt
/// cargo run --features config --example config
/// ```
///
/// # Testing the DefaultHandler
/// ```bash
/// cargo run --features config --example config
/// ```
#[cfg(feature = "config")]
fn main() {
    let config = config::Config::builder()
        .add_source(config::File::new(
            std::env::current_dir()
                .unwrap()
                .join("default.yml")
                .as_path()
                .to_str()
                .unwrap(),
            config::FileFormat::Yaml,
        ))
        .build()
        .unwrap();

    let handler = ConfigHandler::new(Box::new(config)).next(Box::new(
        EnvHandler::new().next(Box::new(
            FileHandler::new(
                std::env::current_dir()
                    .unwrap()
                    .join("verbosity.txt")
                    .as_path()
                    .to_str()
                    .unwrap(),
            )
            .next(Box::new(DefaultHandler::new("trace"))),
        )),
    ));
    // Safe to unwrap since we end the chain with a DefaultHandler which will always return "trace".
    let verbosity = handler.handle_request("verbosity").unwrap();
    println!("verbosity = {}", verbosity);
}

#[cfg(not(feature = "config"))]
fn main() {
    eprintln!("You need to run with the 'config' feature enabled!");
    eprintln!("Try:");
    eprintln!("\tcargo run --features config --example config");
}
