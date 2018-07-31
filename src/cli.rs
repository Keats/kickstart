use clap::{App, Arg, SubCommand, AppSettings};


pub fn build_cli() -> App<'static, 'static> {
    App::new("kickstart")
        .version(crate_version!())
        .author(crate_authors!())
        .about(crate_description!())
        .setting(AppSettings::SubcommandsNegateReqs)
        .arg(
            Arg::with_name("template")
                .required(true)
                .help("Template to use: a local path or a HTTP url pointing to a Git repository")
        )
        .arg(
            Arg::with_name("output-dir")
                .short("o")
                .long("output-dir")
                .takes_value(true)
                .help("Where to output the project: defaults to the current directory")
        )
        .subcommands(vec![
            SubCommand::with_name("validate")
                .about("Validates that a template.toml is valid")
                .arg(
                    Arg::with_name("path")
                        .required(true)
                        .help("The path to the template.toml")
                ),
        ])
}
