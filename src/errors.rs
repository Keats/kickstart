use toml;


error_chain! {
    errors {}

       foreign_links {
        Io(::std::io::Error);
        Toml(toml::de::Error);
    }
}
