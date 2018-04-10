use clap::{App, Arg};


pub fn build_cli() -> App<'static, 'static> {
    App::new("kickstart")
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(
            Arg::with_name("template")
                .required(true)
                .help("Template to use: a local path or a git url")
        )
        .arg(
            Arg::with_name("output-dir")
                .short("o")
                .long("output-dir")
                .takes_value(true)
                .help("Where to output the project: defaults to the current directory")
        )
        .subcommands(vec![])
}
