use cor_args::{DefaultHandler, EnvHandler, FileHandler, Handler};

/// This example can be run multiple ways to test out the Chain of Responsibility.
///
/// # Testing the EnvHandler
/// ```bash
/// verbosity=info cargo run --example simple
/// ```
///
/// # Testing the FileHandler
/// ```bash
/// echo debug > verbosity.txt
/// cargo run --example simple
/// ```
///
/// # Testing the DefaultHandler
/// ```bash
/// cargo run --example simple
/// ```
fn main() {
    let handler = EnvHandler::new().next(Box::new(
        FileHandler::new(
            std::env::current_dir()
                .unwrap()
                .join("verbosity.txt")
                .as_path()
                .to_str()
                .unwrap(),
        )
        .next(Box::new(DefaultHandler::new("trace"))),
    ));
    // Safe to unwrap since we end the chain with a DefaultHandler which will always return "trace".
    let verbosity = handler.handle_request("verbosity").unwrap();
    println!("verbosity = {}", verbosity);
}
