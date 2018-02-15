use clap::{App, Arg};

pub fn build_cli() -> App<'static, 'static> {
    App::new("kickstart")
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .arg(
            Arg::with_name("template")
                .required(true)
                .help("Template to use: a local path or a url")
        )
        .arg(
            Arg::with_name("output_dir")
                .short("o")
                .long("output_dir")
                .takes_value(true)
                .help("Where to output the project: defaults to the current directory")
        )
        .subcommands(vec![])
}
