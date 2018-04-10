# kickstart

A WIP equivalent of [cookiecutter](https://github.com/audreyr/cookiecutter)
in Rust.

## Run on examples

- cargo run -- examples/super-basic
- cargo run -- examples/rust-cli -o clicli
- cargo run -- git@github.com:Keats/kickstart-sample.git -o sample

## Principle

Templates are just moving files/directories from a source to a destination but
being to customise it is essential, otherwise you can just do a `git clone` and be done with it.

In short the most important points are:

- asking the user questions to personalize the result
- work with local folders and remote URLs (git only for now)
- a template engine with whitespace management so the result files look handwritten

Since we are only dealing with files and directories, the tool is completely language-agnostic as well.

It is very largely inspired by [cookiecutter](https://github.com/audreyr/cookiecutter) but trying to
give a slightly better UX by allowing template writers to formulate questions rather than use the variable name
and, later on, to have conditional questions.

For example for a server+frontend template, the questions could look like that:

```text
- Which database do you want to use?
1. Postgres
2. MySQL
3. SQLite
4. None
Please choose from 1, 2, 3, 4 [1]: 1

- Which version of Postgres do you want to use?
1. 10.3
2. 9.6
Please choose from 1, 2 [1]: 1

- How are users going to be authenticated?
1. JWT
2. Passwords
3. None
Please choose from 1, 2 [1]: 1

- Do you want to add Sentry integration? [Y/n]: y

- Is the frontend a SPA? [y/N]: y

- Which JS framework do you want to setup?
1. React
2. Angular
3. Vue
Please choose from 1, 2, 3 [1]: 1

- Do you want to use TypeScript? [Y/n]: y
```

## TODO

- error handling around questions
- generate tmp folder name from URL
- keep questions in order of the template.toml file: https://github.com/alexcrichton/toml-rs/issues/232
- better looking UI (colours, progress, etc)
- potentially conditional questions? some questions could be asked only if a previous value has been set to true
for example
- cache remote repositories?
- Add verbose name for choices?
- make it usable as a library
