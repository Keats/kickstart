use toml::Value;

/// A condition for a question to be asked
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Condition {
    pub name: String,
    pub value: Value,
}

/// A question loaded from TOML
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct Variable {
    /// The variable name in the final context
    pub name: String,
    /// A default value is required
    pub default: Value,
    /// The text asked to the user
    pub prompt: String,
    /// Only for questions with choices
    pub choices: Option<Vec<Value>>,
    /// A regex pattern to validate the input
    pub validation: Option<String>,
    /// Only ask this variable is the condition is true
    pub only_if: Option<Condition>,
}

#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct TemplateDefinition {
    /// Name of the template
    pub name: String,
    /// What this template is about
    pub description: String,
    /// Template version
    pub version: Option<String>,
    /// Version of the kickstart template spec
    pub kickstart_version: usize,
    /// Url of the template
    pub url: Option<String>,
    /// A list of the authors
    #[serde(default)]
    pub authors: Vec<String>,
    /// Some keywords/tags
    #[serde(default)]
    pub keywords: Vec<String>,
    /// Do not copy those directories/files
    pub ignore: Vec<String>,
    /// Do not pass those files through Tera
    /// http://cookiecutter.readthedocs.io/en/latest/advanced/copy_without_render.html
    #[serde(default)]
    pub copy_without_render: Vec<String>,
    /// All the questions for that template
    pub variables: Vec<Variable>,
}


#[cfg(test)]
mod tests {
    use toml;

    use super::*;

    #[test]
    fn can_load_template() {
        let tpl: TemplateDefinition = toml::from_str(r#"
            name = "Test template"
            description = "A description"
            kickstart_version = 1

            [[variables]]
            name = "project_name"
            default = "My project"
            prompt = "What's the name of your project?"

            [[variables]]
            name = "database"
            default = "postgres"
            prompt = "Which database to use?"
            choices = ["postgres", "mysql"]

            [[variables]]
            name = "pg_version"
            prompt = "Which version of Postgres?"
            default = "10.4"
            choices = ["10.4", "9.3"]
            only_if = { name = "database", value = "postgres" }

        "#).unwrap();

        assert_eq!(tpl.variables.len(), 3);
    }
}
