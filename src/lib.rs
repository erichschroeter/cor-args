use serde_json::Value;
use std::borrow::Cow;
use std::env;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;

#[cfg(feature = "clap")]
pub use self::internal_clap::*;
#[cfg(feature = "config")]
pub use self::internal_config::*;

/// A trait for handling requests based on a key.
///
/// This trait provides a mechanism for handling requests by taking a key and
/// returning an associated value wrapped in an `Option`.
pub trait Handler {
    /// Handles a request based on the provided key.
    ///
    /// # Arguments
    ///
    /// * `key` - The key associated with the request.
    ///
    /// # Returns
    ///
    /// An `Option` wrapping a `String` value associated with the key.
    /// If there's no value associated with the key, it should return `None`.
    fn handle_request(&self, key: &str) -> Option<String>;
}

/// A default implementation of the `Handler` trait.
///
/// This struct contains a single `value` that will be returned for any request,
/// regardless of the provided key.
///
/// # Examples
///
/// ```
/// use cor_args::{DefaultHandler, Handler};
///
/// // Create a new DefaultHandler for a specific value
/// let handler = DefaultHandler::new("some_value");
///
/// // Add a fallback handler
/// //let handler = handler.next(some_other_handler.into());
///
/// // Handle a configuration request
/// let value = handler.handle_request("some_key");
/// ```
pub struct DefaultHandler {
    value: String,
}

impl DefaultHandler {
    /// Creates a new `DefaultHandler` with the specified value.
    ///
    /// # Arguments
    ///
    /// * `value` - The value to be returned for any request.
    ///
    /// # Examples
    ///
    /// ```
    /// use cor_args::DefaultHandler;
    ///
    /// let handler = DefaultHandler::new("some_value");
    /// ```
    #[allow(dead_code)]
    pub fn new(value: &str) -> Self {
        DefaultHandler {
            value: String::from(value),
        }
    }
}

impl Handler for DefaultHandler {
    /// Always returns the stored value, regardless of the key.
    ///
    /// This implementation ignores the provided key and always returns the
    /// value stored in the `DefaultHandler`.
    fn handle_request(&self, _key: &str) -> Option<String> {
        Some(self.value.clone())
    }
}

impl Into<Box<dyn Handler>> for DefaultHandler {
    fn into(self) -> Box<dyn Handler> {
        Box::new(self)
    }
}

#[cfg(feature = "clap")]
pub mod internal_clap {
    use super::*;
    use clap::ArgMatches;
    /// A handler for managing command-line arguments.
    ///
    /// This struct is responsible for handling command-line arguments passed to the application.
    /// If a value for a given key is not found in the arguments, it delegates the request to the
    /// next handler (if provided).
    ///
    /// # Examples
    ///
    /// ```
    /// use cor_args::{ArgHandler, Handler};
    ///
    /// // Create a simple `clap` command
    /// let args = clap::Command::new("myapp")
    ///     .arg(clap::Arg::new("example").long("example"))
    ///     .get_matches();
    ///
    /// // Create a new ArgHandler for a `clap::ArgMatches`
    /// let handler = ArgHandler::new(&args);
    ///
    /// // Add a fallback handler
    /// //let handler = handler.next(some_other_handler.into());
    ///
    /// // Handle a configuration request matching the `clap::Arg` name
    /// let value = handler.handle_request("example");
    /// ```
    pub struct ArgHandler<'a> {
        /// Parsed command-line arguments.
        args: &'a ArgMatches,
        /// An optional next handler to delegate requests if this handler can't fulfill them.
        next: Option<Box<dyn Handler>>,
    }

    impl<'a> ArgHandler<'a> {
        /// Creates a new `ArgHandler` with the specified arguments.
        ///
        /// # Arguments
        ///
        /// * `args` - The parsed command-line arguments.
        ///
        /// # Examples
        ///
        /// ```
        /// use cor_args::ArgHandler;
        ///
        /// let args = clap::Command::new("myapp")
        ///     .arg(clap::Arg::new("config").long("some-option"))
        ///     .get_matches();
        ///
        /// let handler = ArgHandler::new(&args);
        /// ```
        #[allow(dead_code)]
        pub fn new(args: &'a ArgMatches) -> Self {
            ArgHandler { args, next: None }
        }

        #[allow(dead_code)]
        pub fn next(mut self, handler: Box<dyn Handler>) -> Self {
            self.next = Some(handler);
            self
        }
    }

    impl<'a> Handler for ArgHandler<'a> {
        /// Retrieves a value for the specified key from the command-line arguments.
        ///
        /// If the key is not found in the arguments, and if a next handler is provided, it delegates the request
        /// to the next handler. If there's no next handler or if the key is not found in both the arguments and
        /// the next handler, it returns `None`.
        ///
        /// # Arguments
        ///
        /// * `key` - The key for which the value needs to be retrieved.
        ///
        /// # Returns
        ///
        /// An `Option` containing the value associated with the key, or `None` if the key is not found.
        fn handle_request(&self, key: &str) -> Option<String> {
            if let Ok(value) = self.args.try_get_one::<String>(key) {
                if let Some(value) = value {
                    return Some(value.clone());
                }
            }
            if let Some(next_handler) = &self.next {
                return next_handler.handle_request(key);
            }
            None
        }
    }

    impl<'a> Into<Box<dyn Handler + 'a>> for ArgHandler<'a> {
        fn into(self) -> Box<dyn Handler + 'a> {
            Box::new(self)
        }
    }
}

/// A handler for retrieving values from environment variables.
///
/// This struct is responsible for handling requests by checking for the existence of
/// an environment variable corresponding to the provided key. If the environment variable
/// is not found, it delegates the request to the next handler (if provided).
///
/// # Examples
///
/// ```
/// use cor_args::{EnvHandler, Handler};
///
/// // Create a new EnvHandler specifying a prefix for environment variables
/// let handler = EnvHandler::new().prefix("MYAPP_");
///
/// // Add a fallback handler
/// //let handler = handler.next(some_other_handler.into());
///
/// // Handle a configuration request matching `MYAPP_some_key`
/// let value = handler.handle_request("some_key");
/// ```
pub struct EnvHandler<'a> {
    /// A prefix to prepend to the key passed to `handle_request()`.
    prefix: Option<Cow<'a, str>>,
    /// An optional next handler to delegate requests if this handler can't fulfill them.
    next: Option<Box<dyn Handler>>,
}

impl<'a> EnvHandler<'a> {
    /// Creates a new `EnvHandler`.
    ///
    /// # Arguments
    ///
    /// * `prefix` - An optional prefix to which requests will prepend when `handle_request()` is executed.` If `None`, an empty string is assigned.
    ///
    /// # Examples
    ///
    /// ```
    /// use cor_args::EnvHandler;
    ///
    /// let handler = EnvHandler::new();
    /// ```
    #[allow(dead_code)]
    pub fn new() -> Self {
        EnvHandler {
            prefix: None,
            next: None,
        }
    }

    #[allow(dead_code)]
    pub fn next(mut self, handler: Box<dyn Handler>) -> Self {
        self.next = Some(handler);
        self
    }

    #[allow(dead_code)]
    pub fn prefix<S>(mut self, prefix: S) -> Self
    where
        S: Into<Cow<'a, str>>,
    {
        self.prefix = Some(prefix.into());
        self
    }
}

impl<'a> Handler for EnvHandler<'a> {
    /// Retrieves a value for the specified key from the environment variables.
    ///
    /// If the environment variable corresponding to the key is not found, and if a next handler is provided,
    /// it delegates the request to the next handler. If there's no next handler or if the key is not found
    /// both in the environment and the next handler, it returns `None`.
    ///
    /// # Arguments
    ///
    /// * `key` - The key for which the value needs to be retrieved from environment variables.
    ///
    /// # Returns
    ///
    /// An `Option` containing the value associated with the key, or `None` if the key is not found.
    fn handle_request(&self, key: &str) -> Option<String> {
        if let Some(prefix) = &self.prefix {
            let key = format!("{prefix}{key}");
            if let Ok(value) = env::var(key) {
                return Some(value);
            }
        } else {
            if let Ok(value) = env::var(key) {
                return Some(value);
            }
        }
        if let Some(next_handler) = &self.next {
            return next_handler.handle_request(key);
        }
        None
    }
}

impl<'a> Into<Box<dyn Handler + 'a>> for EnvHandler<'a> {
    fn into(self) -> Box<dyn Handler + 'a> {
        Box::new(self)
    }
}

/// A handler for retrieving values from a file.
///
/// This struct is responsible for handling requests by checking for values within a specified file.
///
/// # Examples
///
/// ```
/// use cor_args::{FileHandler, Handler};
///
/// // Create a new FileHandler specifying a path to a file.
/// let handler = FileHandler::new("/path/to/file");
///
/// // Add a fallback handler
/// //let handler = handler.next(some_other_handler.into());
///
/// // Handle a configuration request returning contents of `/path/to/file`
/// let value = handler.handle_request("");
/// ```
pub struct FileHandler {
    /// Path to the file from which values are to be retrieved.
    file_path: PathBuf,
    /// An optional next handler to delegate requests if this handler can't fulfill them.
    next: Option<Box<dyn Handler>>,
}

impl FileHandler {
    /// Creates a new `FileHandler` with the specified file path.
    ///
    /// # Arguments
    ///
    /// * `file_path` - The path to the file from which values are to be retrieved.
    ///
    /// # Examples
    ///
    /// ```
    /// use cor_args::FileHandler;
    ///
    /// let handler = FileHandler::new("/path/to/file");
    /// ```
    #[allow(dead_code)]
    pub fn new<P>(file_path: P) -> Self
    where
        P: Into<PathBuf>,
    {
        FileHandler {
            file_path: file_path.into(),
            next: None,
        }
    }

    #[allow(dead_code)]
    pub fn next(mut self, handler: Box<dyn Handler>) -> Self {
        self.next = Some(handler);
        self
    }
}

impl Handler for FileHandler {
    /// Retrieves content from the specified file.
    ///
    /// This implementation attempts to read content from the file specified by `file_path`.
    /// If reading fails, and if a next handler is provided, it delegates the request
    /// to the next handler. If there's no next handler or if the file reading fails,
    /// it returns `None`.
    ///
    /// # Arguments
    ///
    /// * `key` - The key for which the value needs to be retrieved. (Note: The `key` is currently not used directly, just passed on to the next handler.)
    ///
    /// # Returns
    ///
    /// An `Option` containing the contents of the file, or `None` if the key is not found.
    fn handle_request(&self, key: &str) -> Option<String> {
        if let Ok(mut file) = File::open(&self.file_path) {
            let mut content = String::new();
            if let Ok(_byte_count) = file.read_to_string(&mut content) {
                return Some(content);
            }
        }
        if let Some(next_handler) = &self.next {
            return next_handler.handle_request(key);
        }
        None
    }
}

impl Into<Box<dyn Handler>> for FileHandler {
    fn into(self) -> Box<dyn Handler> {
        Box::new(self)
    }
}

/// A handler for retrieving values from a specified JSON file.
///
/// This struct is responsible for handling requests by reading content from the file
/// specified in the underlying `FileHandler`, and then searching for a specific key
/// within the parsed JSON structure. If the key is not found in the JSON structure,
/// it delegates the request to the next handler (if provided).
///
/// ```
/// use cor_args::{JSONFileHandler, Handler};
///
/// // Create a new JSONFileHandler specifying a path to a file.
/// let handler = JSONFileHandler::new("file.json");
///
/// // Add a fallback handler
/// //let handler = handler.next(some_other_handler.into());
///
/// // Handle a configuration request matching a `"some_key"` within `file.json`
/// let value = handler.handle_request("some_key");
/// ```
pub struct JSONFileHandler {
    /// Underlying file handler used to read content from the specified file.
    file_handler: FileHandler,
}

impl JSONFileHandler {
    /// Creates a new `JSONFileHandler` with the specified file path.
    ///
    /// # Arguments
    ///
    /// * `file_path` - The path to the JSON file from which values are to be retrieved.
    ///
    /// # Examples
    ///
    /// ```
    /// use cor_args::FileHandler;
    ///
    /// let handler = FileHandler::new("file.json");
    /// ```
    #[allow(dead_code)]
    pub fn new<P>(file_path: P) -> Self
    where
        P: Into<PathBuf>,
    {
        JSONFileHandler {
            file_handler: FileHandler::new(file_path),
        }
    }

    #[allow(dead_code)]
    pub fn next(mut self, handler: Box<dyn Handler>) -> Self {
        self.file_handler.next = Some(handler);
        self
    }

    /// Recursively searches for a key within the parsed JSON structure.
    ///
    /// # Arguments
    ///
    /// * `json_value` - The current JSON value being inspected.
    /// * `key` - The key for which the value needs to be retrieved.
    ///
    /// # Returns
    ///
    /// If found, returns an `Option` wrapping a `String` value associated with the key.
    /// Otherwise, returns `None`.
    pub fn find_key_recursive(json_value: &Value, key: &str) -> Option<String> {
        match json_value {
            Value::Object(map) => {
                if let Some(value) = map.get(key) {
                    match value {
                        serde_json::Value::String(value) => {
                            return Some(value.as_str().to_string())
                        }
                        _ => return Some(value.to_string()),
                    }
                }
                for (_, value) in map.iter() {
                    if let Some(found) = Self::find_key_recursive(value, key) {
                        return Some(found);
                    }
                }
            }
            Value::Array(arr) => {
                for value in arr.iter() {
                    if let Some(found) = Self::find_key_recursive(value, key) {
                        return Some(found);
                    }
                }
            }
            _ => {}
        }
        None
    }
}

impl Handler for JSONFileHandler {
    /// Retrieves a value for the specified key from the JSON file.
    ///
    /// This implementation attempts to read content from the file specified in the underlying `FileHandler`,
    /// parses the content as JSON, and then searches for the specified key within the parsed JSON structure.
    /// If the key is not found in the JSON structure, and if a next handler is provided, it delegates the request
    /// to the next handler. If there's no next handler, or if the key is not found in both the JSON structure
    /// and the next handler, it returns `None`.
    ///
    /// # Arguments
    ///
    /// * `key` - The key for which the value needs to be retrieved from the JSON file.
    ///
    /// # Returns
    ///
    /// An `Option` containing the value associated with the key, or `None` if the key is not found.
    fn handle_request(&self, key: &str) -> Option<String> {
        if let Some(file_data) = self.file_handler.handle_request(key) {
            if let Ok(parsed_json) = serde_json::from_str::<Value>(&file_data) {
                if let Some(value) = Self::find_key_recursive(&parsed_json, key) {
                    return Some(value);
                }
            } else {
                if let Some(next_handler) = &self.file_handler.next {
                    return next_handler.handle_request(key);
                }
            }
        }
        None
    }
}

impl Into<Box<dyn Handler>> for JSONFileHandler {
    fn into(self) -> Box<dyn Handler> {
        Box::new(self)
    }
}

#[cfg(feature = "config")]
pub mod internal_config {
    use super::*;
    use config::Config;
    /// A configuration file handler for reading key-value pairs from a file.
    ///
    /// The `ConfigHandler` is used to read configuration data from a file and provide it
    /// as key-value pairs. It supports chaining multiple handlers for fallback behavior.
    ///
    /// # Examples
    ///
    /// ```
    /// use cor_args::{ConfigHandler, Handler};
    ///
    /// // Example YAML file
    /// // ---
    /// // test_obj:
    /// //     some_key: "test_val"
    ///
    /// let config = config::Config::builder().build().unwrap();
    /// //    .add_source(config::File::new("/path/to/file",
    /// //        config::FileFormat::Yaml,
    /// //    ))
    /// //    .build()
    /// //    .unwrap();
    ///
    /// // Create a new ConfigHandler for a specific config::Config instance
    /// let handler = ConfigHandler::new(Box::new(config));
    ///
    /// // Add a fallback handler
    /// //let handler = handler.next(some_other_handler.into());
    ///
    /// // Handle a configuration request
    /// let value = handler.handle_request("some_key");
    /// ```
    pub struct ConfigHandler {
        /// The Config instance ultimately being queried.
        config: Box<config::Config>,
        next: Option<Box<dyn Handler>>,
    }

    impl ConfigHandler {
        /// Create a new `ConfigHandler` for the specified file path.
        ///
        /// # Parameters
        ///
        /// - `config`: A `config::Config` reference.
        ///
        /// # Returns
        ///
        /// A new `ConfigHandler` instance.
        ///
        /// # Examples
        ///
        /// ```
        /// use cor_args::ConfigHandler;
        ///
        /// let handler = ConfigHandler::new(Box::new(config::Config::builder().build().unwrap()));
        /// ```
        #[allow(dead_code)]
        pub fn new(config: Box<Config>) -> Self {
            ConfigHandler { config, next: None }
        }

        #[allow(dead_code)]
        pub fn next(mut self, handler: Box<dyn Handler>) -> Self {
            self.next = Some(handler);
            self
        }

        /// Recursively searches for a key within the parsed Config structure.
        ///
        /// # Arguments
        ///
        /// * `config_value` - The current Config value being inspected.
        /// * `key` - The key for which the value needs to be retrieved.
        ///
        /// # Returns
        ///
        /// If found, returns an `Option` wrapping a `String` value associated with the key.
        /// Otherwise, returns `None`.
        pub fn find_key_recursive(config_value: &config::Value, key: &str) -> Option<String> {
            match &config_value.kind {
                config::ValueKind::Table(map) => {
                    if let Some(value) = map.get(key) {
                        match &value.kind {
                            config::ValueKind::String(value) => {
                                return Some(value.as_str().to_string())
                            }
                            _ => return Some(value.to_string()),
                        }
                    }
                    for (_, value) in map.iter() {
                        if let Some(found) = Self::find_key_recursive(value, key) {
                            return Some(found);
                        }
                    }
                }
                config::ValueKind::Array(arr) => {
                    for value in arr.iter() {
                        if let Some(found) = Self::find_key_recursive(value, key) {
                            return Some(found);
                        }
                    }
                }
                _ => {}
            }
            None
        }
    }

    impl Handler for ConfigHandler {
        /// Handle a configuration request and return the value associated with the provided key.
        ///
        /// This method attempts to read the configuration file and retrieve the value associated
        /// with the given key. If the key is not found, it may delegate the request to a fallback
        /// handler if one is defined.
        ///
        /// # Parameters
        ///
        /// - `key`: A string representing the configuration key to retrieve.
        ///
        /// # Returns
        ///
        /// An `Option` containing the value associated with the key, or `None` if the key is not found.
        fn handle_request(&self, key: &str) -> Option<String> {
            if let Ok(parsed_config) = self.config.clone().try_deserialize::<config::Value>() {
                if let Some(value) = Self::find_key_recursive(&parsed_config, key) {
                    return Some(value);
                }
            }
            if let Some(next_handler) = &self.next {
                return next_handler.handle_request(key);
            }
            None
        }
    }

    impl Into<Box<dyn Handler>> for ConfigHandler {
        fn into(self) -> Box<dyn Handler> {
            Box::new(self)
        }
    }

    impl From<Result<config::Config, config::ConfigError>> for ConfigHandler {
        fn from(value: Result<config::Config, config::ConfigError>) -> Self {
            if let Ok(config) = value {
                ConfigHandler::new(Box::new(config))
            } else {
                panic!("Failed to convert into a Config")
            }
        }
    }

    impl From<config::Config> for ConfigHandler {
        fn from(value: config::Config) -> Self {
            ConfigHandler::new(Box::new(value))
        }
    }
}

#[cfg(test)]
mod tests {
    use std::io::Write;
    use tempfile::NamedTempFile;

    use super::*;

    #[cfg(feature = "clap")]
    #[test]
    fn test_clap_features_chain_of_responsibility() {
        env::set_var("TEST_KEY", "EnvHandler");
        let args = clap::Command::new("test_app")
            .arg(clap::Arg::new("example").long("example"))
            .get_matches_from(vec!["test_app", "--example", "ArgHandler"]);
        let temp_dir = tempfile::tempdir().unwrap();
        // Don't create the temporary file so the chain keeps going to the end for this test.
        let raw_file = temp_dir.path().join("should-not-exist.txt");
        let mut json_file = NamedTempFile::new().unwrap();
        writeln!(json_file, r#"{{"test_key": "JSONFileHandler"}}"#).unwrap();

        let handler = ArgHandler::new(&args).next(Box::new(
            EnvHandler::new().next(Box::new(
                FileHandler::new(raw_file.as_path().to_str().unwrap())
                    .next(Box::new(JSONFileHandler::new(
                        json_file.path().to_str().unwrap(),
                    )))
                    .next(Box::new(DefaultHandler::new("DefaultHandler"))),
            )),
        ));
        let actual = handler.handle_request("");
        assert_eq!(actual, Some("DefaultHandler".to_string()));
    }

    #[test]
    fn test_default_features_chain_of_responsibility() {
        env::set_var("TEST_KEY", "EnvHandler");
        let temp_dir = tempfile::tempdir().unwrap();
        // Don't create the temporary file so the chain keeps going to the end for this test.
        let raw_file = temp_dir.path().join("should-not-exist.txt");
        let mut json_file = NamedTempFile::new().unwrap();
        writeln!(json_file, r#"{{"test_key": "JSONFileHandler"}}"#).unwrap();

        let handler = EnvHandler::new().next(Box::new(
            FileHandler::new(raw_file.as_path().to_str().unwrap())
                .next(Box::new(JSONFileHandler::new(
                    json_file.path().to_str().unwrap(),
                )))
                .next(Box::new(DefaultHandler::new("DefaultHandler"))),
        ));
        let actual = handler.handle_request("");
        assert_eq!(actual, Some("DefaultHandler".to_string()));
    }

    #[cfg(feature = "config")]
    #[test]
    fn test_config_features_chain_of_responsibility() {
        env::set_var("UNUSED", "EnvHandler");
        let mut temp_file = tempfile::Builder::new().suffix(".yml").tempfile().unwrap();
        let expected = r#"
        ---
        test_obj:
            test_key: "test_val"
        "#;
        writeln!(temp_file, "{}", unindent::unindent(expected)).unwrap();

        let config = config::Config::builder()
            .add_source(config::File::new(
                temp_file.path().to_str().unwrap(),
                config::FileFormat::Yaml,
            ))
            .build()
            .unwrap();

        let handler = EnvHandler::new().next(Box::new(ConfigHandler::new(Box::new(config))));
        // let handler = EnvHandler::new().next(Box::<ConfigHandler>::new(config.into()));
        let actual = handler.handle_request("test_key");
        assert_eq!(actual, Some("test_val".to_string()));
    }

    mod default_handler {
        use super::*;

        #[test]
        fn test_retrieves_set_value() {
            let handler = DefaultHandler::new("TEST_VAL");
            let actual = handler.handle_request("");
            assert_eq!(actual, Some("TEST_VAL".to_string()));
        }
    }

    mod env_handler {
        use super::*;

        #[test]
        fn test_retrieves_set_value_without_prefix() {
            env::set_var("TEST_KEY", "test_value");
            let handler = EnvHandler::new();
            let actual = handler.handle_request("TEST_KEY");
            assert_eq!(actual, Some("test_value".to_string()));
        }

        #[test]
        fn test_retrieves_set_value_with_prefix() {
            env::set_var("TEST_KEY", "test_value");
            let handler = EnvHandler::new().prefix("TEST_");
            let actual = handler.handle_request("KEY");
            assert_eq!(actual, Some("test_value".to_string()));
        }

        #[test]
        fn test_returns_none_for_unset_value() {
            env::remove_var("UNSET_KEY"); // Ensure the variable is not set
            let handler = EnvHandler::new();
            let actual = handler.handle_request("UNSET_KEY");
            assert_eq!(actual, None);
        }

        #[test]
        fn test_next_handler_called() {
            env::remove_var("UNSET_KEY"); // Ensure the variable is not set
            let next_handler = Box::new(DefaultHandler::new("DEFAULT_VALUE"));
            let handler = EnvHandler::new().next(next_handler);
            let actual = handler.handle_request("UNSET_KEY");
            assert_eq!(actual, Some("DEFAULT_VALUE".to_string()));
        }
    }

    #[cfg(feature = "clap")]
    mod arg_handler {
        use clap::Arg;

        use super::*;

        #[test]
        fn test_retrieves_set_value() {
            let args = clap::Command::new("test_app")
                .arg(Arg::new("example").long("example"))
                .get_matches_from(vec!["test_app", "--example", "test_value"]);

            let handler = ArgHandler::new(&args);
            let result = handler.handle_request("example");
            assert_eq!(result, Some("test_value".to_string()));
        }

        #[test]
        fn test_returns_none_for_unset_value() {
            let args = clap::Command::new("test_app")
                .arg(Arg::new("example").long("example"))
                .get_matches_from(vec!["test_app"]);

            let handler = ArgHandler::new(&args);
            let result = handler.handle_request("example");
            assert_eq!(result, None);
        }

        #[test]
        fn test_next_handler_called() {
            let args = clap::Command::new("test_app")
                .arg(Arg::new("example").long("example"))
                .get_matches_from(vec!["test_app"]);
            let next_handler = Box::new(DefaultHandler::new("DEFAULT_VALUE"));
            let handler = ArgHandler::new(&args).next(next_handler);
            let actual = handler.handle_request("example");
            assert_eq!(actual, Some("DEFAULT_VALUE".to_string()));
        }
    }

    mod file_handler {
        use std::io::Write;
        use tempfile::NamedTempFile;

        use super::*;

        #[test]
        fn test_retrieves_set_value() {
            let mut temp_file = NamedTempFile::new().unwrap();
            writeln!(temp_file, "test_content").unwrap();

            let handler = FileHandler::new(temp_file.path().to_str().unwrap());
            let result = handler.handle_request(""); // key is not used in this handler
            assert_eq!(result, Some("test_content\n".to_string()));
        }

        #[test]
        fn test_returns_none_for_nonexistent_file() {
            let handler = FileHandler::new("");
            let result = handler.handle_request("example");
            assert_eq!(result, None);
        }

        #[test]
        fn test_next_handler_called() {
            let next_handler = Box::new(DefaultHandler::new("DEFAULT_VALUE"));
            let handler = FileHandler::new("").next(next_handler);
            let actual = handler.handle_request("example");
            assert_eq!(actual, Some("DEFAULT_VALUE".to_string()));
        }
    }

    mod json_file_handler {
        use std::io::Write;
        use tempfile::NamedTempFile;

        use super::*;

        #[test]
        fn test_retrieves_set_value_number() {
            let mut temp_file = NamedTempFile::new().unwrap();
            writeln!(temp_file, r#"{{"test_key": 123}}"#).unwrap();

            let handler = JSONFileHandler::new(temp_file.path().to_str().unwrap());
            let actual = handler.handle_request("test_key"); // key is not used in this handler
            assert_eq!(actual, Some("123".to_string()));
        }

        #[test]
        fn test_retrieves_set_value_string() {
            let mut temp_file = NamedTempFile::new().unwrap();
            writeln!(temp_file, r#"{{"test_key": "example"}}"#).unwrap();

            let handler = JSONFileHandler::new(temp_file.path().to_str().unwrap());
            let actual = handler.handle_request("test_key"); // key is not used in this handler
            assert_eq!(actual, Some("example".to_string()));
        }

        #[test]
        fn test_retrieves_set_value_nested_object() {
            let mut temp_file = NamedTempFile::new().unwrap();
            writeln!(temp_file, r#"{{"test_obj": {{"test_key": "example"}} }}"#).unwrap();

            let handler = JSONFileHandler::new(temp_file.path().to_str().unwrap());
            let actual = handler.handle_request("test_key"); // key is not used in this handler
            assert_eq!(actual, Some("example".to_string()));
        }

        #[test]
        fn test_retrieves_set_value_in_array() {
            let mut temp_file = NamedTempFile::new().unwrap();
            writeln!(temp_file, r#"[{{"test_key": "example"}}]"#).unwrap();

            let handler = JSONFileHandler::new(temp_file.path().to_str().unwrap());
            let actual = handler.handle_request("test_key"); // key is not used in this handler
            assert_eq!(actual, Some("example".to_string()));
        }

        #[test]
        fn test_returns_none_for_nonexistent_file() {
            let handler = JSONFileHandler::new("");
            let result = handler.handle_request("example");
            assert_eq!(result, None);
        }

        #[test]
        fn test_next_handler_called() {
            let next_handler = Box::new(DefaultHandler::new("DEFAULT_VALUE"));
            let handler = JSONFileHandler::new("").next(next_handler);
            let actual = handler.handle_request("example");
            assert_eq!(actual, Some("DEFAULT_VALUE".to_string()));
        }
    }

    #[cfg(feature = "config")]
    mod config_handler {
        use config::Config;
        use std::io::Write;
        use tempfile::Builder;
        use unindent::unindent;

        use super::*;

        #[test]
        fn test_retrieves_set_value_number_as_yaml() {
            let mut temp_file = Builder::new().suffix(".yaml").tempfile().unwrap();
            let expected = r#"
            ---
            test_key: 123
            "#;
            writeln!(temp_file, "{}", unindent(expected)).unwrap();
            let config = config::Config::builder()
                .add_source(config::File::new(
                    temp_file.path().to_str().unwrap(),
                    config::FileFormat::Yaml,
                ))
                .build()
                .unwrap();

            let handler = ConfigHandler::new(Box::new(config));
            let actual = handler.handle_request("test_key");
            assert_eq!(actual, Some("123".to_string()));
        }

        #[test]
        fn test_retrieves_set_value_string_as_yaml() {
            let mut temp_file = Builder::new().suffix(".yaml").tempfile().unwrap();
            let expected = r#"
            ---
            test_key: "example"
            "#;
            writeln!(temp_file, "{}", unindent(expected)).unwrap();
            let config = config::Config::builder()
                .add_source(config::File::new(
                    temp_file.path().to_str().unwrap(),
                    config::FileFormat::Yaml,
                ))
                .build()
                .unwrap();

            let handler = ConfigHandler::new(Box::new(config));
            let actual = handler.handle_request("test_key");
            assert_eq!(actual, Some("example".to_string()));
        }

        #[test]
        fn test_retrieves_set_value_nested_object() {
            let mut temp_file = Builder::new().suffix(".yaml").tempfile().unwrap();
            let expected = r#"
            ---
            test_obj:
                test_key: "test_val"
            "#;
            writeln!(temp_file, "{}", unindent(expected)).unwrap();
            let config = config::Config::builder()
                .add_source(config::File::new(
                    temp_file.path().to_str().unwrap(),
                    config::FileFormat::Yaml,
                ))
                .build()
                .unwrap();

            let handler = ConfigHandler::new(Box::new(config));
            let actual = handler.handle_request("test_key");
            assert_eq!(actual, Some("test_val".to_string()));
        }

        #[test]
        fn test_next_handler_called() {
            let config = Config::default();
            let next_handler = Box::new(DefaultHandler::new("DEFAULT_VALUE"));
            let handler = ConfigHandler::new(Box::new(config)).next(next_handler);
            let actual = handler.handle_request("example");
            assert_eq!(actual, Some("DEFAULT_VALUE".to_string()));
        }
    }
}
