#[cfg(feature = "clap")]
use cor_args::{ArgHandler, DefaultHandler, EnvHandler, FileHandler, Handler};

/// This example can be run multiple ways to test out the Chain of Responsibility.
///
/// # Testing the ArgHandler
/// ```bash
/// cargo run --example clap -- --verbosity debug
/// ```
///
/// # Testing the EnvHandler
/// ```bash
/// verbosity=info cargo run --example clap
/// ```
///
/// # Testing the FileHandler
/// ```bash
/// echo debug > verbosity.txt
/// cargo run --example clap
/// ```
///
/// # Testing the DefaultHandler
/// ```bash
/// cargo run --example clap
/// ```
#[cfg(feature = "clap")]
fn main() {
    let args = clap::Command::new("test_app")
        .arg(clap::Arg::new("verbosity").long("verbosity"))
        .get_matches();

    let handler = ArgHandler::new(&args).next(Box::new(
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

#[cfg(not(feature = "clap"))]
fn main() {
    eprintln!("You need to run with the 'clap' feature enabled!");
    eprintln!("Try:");
    eprintln!("\tcargo run --features clap --example clap");
}
