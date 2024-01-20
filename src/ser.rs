pub struct Config {
    pub indent: usize,
    pub trailing_comma: bool,
    pub validate: bool,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            indent: 2,
            trailing_comma: true,
            validate: true,
        }
    }
}

struct BibtexWriter {
    config: &'static Config,
}

// implement map serialization by holding a temporary buffer with the fields, and then flush when
// the fields are written. But what about corresponding values?
