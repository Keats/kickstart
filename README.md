# kickstart

A CLI tool to easily get a new project up and running by using pre-made templates.
This is a slightly more powerful version of an equivalent tool in Python, [cookiecutter](https://github.com/audreyr/cookiecutter).

## Installation

Currently, `kickstart` is available only through crates.io:

```bash
$ cargo install kickstart
```

## Features

- Cross-platform: Windows, Mac and Linux supported
- Single binary: no need to install a virtualenv or anything else
- Templaces can be made for any kind of projects, not limited to Rust
- Simple CLI usage: only one command
- Directory names and filenames can be templated: `{{ repo_name }}/{{author}}.md` is a valid path
- All templating done through [Tera](https://tera.netlify.com/docs/installation/)
- Choose your own adventure: supports conditional questions based on previous answers
- Can load templates from a local directory or from a Git repository
- Has conditional cleanup to not let irrelevant files in the projects after generation

## Try it out

```bash
# From the root of this repo
$ kickstart examples/super-basic
$ kickstart examples/complex -o Hello
# Anywhere
$ kickstart https://github.com/Keats/kickstart-sample -o sample
```

## Creating your own template
Creating a template is fairly simple: create files and then just add a `template.toml` in the root folder. Here is a description of all the fields available in it:


```toml
# Required, name of the template
name = "Django"
# Optional, longer form description
description = "A fully-featured Django template"
# Required, the version of the kickstart schema, currently only `1` is used
kickstart_version = 1
# Optional, the URL of the template
url = "https://google.com"
# Optional, a list of authors for this template
authors = [

]
# Optional, a list of keywords for this template
kewyords = [

]
# Optional, those files will NOT be copied over when generating the template
# Use it to remove template-specific like its CI or its README/docs
ignore = [
    "README.md",
    "CONTRIBUTING.md",
    ".travis.yml",
    "docs",
]
# Optional, a list of patterns. All files matching one of the patterns will
# be copied over without going through Tera.
# Use it for files that contain syntax similar to Tera for example
copy_without_render = [
    "*.html",
]

# Optional, a list of cleanup actions to do.
# All paths listed will be deleted if the `name` has the value `value` after
# the questions have been answered and the project generated.
cleanup = [
    { name = "spa", value = true, paths = ["{{ project_name | slugify }}/templates/"]},
    { name = "auth_method", value = "none", paths = ["{{ project_name | slugify }}/docs/auth.md"]},
]

# A list of variables, the schema is explained in detail below
[[variables]]
name = "project_name"
default = "my-project"
prompt = "What is the name of this project?"
validation = "^([a-zA-Z][a-zA-Z0-9_-]+)$"

[[variables]]
name = "database"
default = "postgres"
prompt = "Which database do you want to use?"
choices = ["postgres", "mysql", "sqlite"]

[[variables]]
name = "pg_version"
default = "10.4"
prompt = "Which version of Postgres?"
choices =  [
    "10.4",
    "10.3",
    "10.2",
    "10.1",
    "9.6",
    "9.5",
    "9.4",
    "9.3",
]
only_if = { name = "database", value = "postgres" }

[[variables]]
name = "auth_method"
default = "jwt"
prompt = "How are users going to be authenticated?"
choices = ["jwt", "sessions", "none"]

[[variables]]
name = "sentry"
default = true
prompt = "Do you want to add Sentry integration?"

[[variables]]
name = "spa"
default = false
prompt = "Is the frontend a SPA?"

[[variables]]
name = "js_framework"
default = "React"
prompt = "Which JS framework do you want to setup?"
choices =  [
    "React",
    "Angular",
    "Vue",
    "None",
]
only_if = { name = "spa", value = true }

[[variables]]
name = "typescript"
default = true
prompt = "Do you want to use TypeScript?"
only_if = { name = "spa", value = true }

```

A variable has the following required fields:

- `name`: the name of the variable in Tera context
- `default`: the default value for that question, `kickstart` uses that to deduce the type of that value (only string, bool and integer are currently supported)
- `prompt`: the text to display to the user

And two more optional fields:

- `choices`: a list of potential values, `kickstart` will make the user pick one
- `only_if`: this question will only be asked if the variable `name` has the value `value`
- `validation`: a Regex pattern to check when getting a string value

## List of templates

None for now.

## Changelog

### 0.1.1 (2018-07-31)

- Add optional `validation` field to validate a string against a regex
- Add colours and boldness to CLI
- Use `git` command rather than git2 crate to avoid some build issues
- Add `cleanup` field to template definition for post-generation cleanup
- Add `validate` command to diagnose errors in a `template.toml` file
