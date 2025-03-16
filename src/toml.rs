//! Anza CI configuration TOML file.

use {
    crate::parser::{map_of_keyed_elements, TomlNamedElement},
    serde::Deserialize,
    std::{collections::HashMap, process::Command},
};

#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
struct Args(Vec<String>);

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Template {
    name: String,
    args: Option<Args>,
    extra_args: Option<Args>,
    subcommand: Option<String>,
    toolchain: Option<String>,
}

impl TomlNamedElement for Template {
    fn name(&self) -> &String {
        &self.name
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Alias {
    name: String,
    args: Option<Args>,
    extra_args: Option<Args>,
    manifest: Option<String>,
    subcommand: Option<String>,
    template: Option<String>,
    toolchain: Option<String>,
}

impl TomlNamedElement for Alias {
    fn name(&self) -> &String {
        &self.name
    }
}

#[derive(Debug, Deserialize)]
pub struct Toml {
    #[serde(rename = "template", deserialize_with = "map_of_keyed_elements")]
    templates: HashMap<String, Template>,
    #[serde(rename = "alias", deserialize_with = "map_of_keyed_elements")]
    aliases: HashMap<String, Alias>,
}

impl Toml {
    pub fn parse(file_contents: &str) -> Result<Self, toml::de::Error> {
        toml::de::from_str(file_contents)
    }

    pub fn compile_alias_command(&self, alias_name: &str) -> Option<Command> {
        let alias = self.aliases.get(alias_name)?;

        let mut cmd = Command::new("cargo");

        match alias.template.as_ref().and_then(|t| self.templates.get(t)) {
            Some(template) => {
                // Alias takes precedence over template.

                if let Some(toolchain) = alias.toolchain.as_ref() {
                    cmd.arg(format!("+{}", toolchain));
                } else if let Some(toolchain) = template.toolchain.as_ref() {
                    cmd.arg(format!("+{}", toolchain));
                }

                if let Some(subcommand) = alias.subcommand.as_ref() {
                    cmd.arg(subcommand);
                } else if let Some(subcommand) = template.subcommand.as_ref() {
                    cmd.arg(subcommand);
                }

                if let Some(args) = alias.args.as_ref() {
                    cmd.args(&args.0);
                } else if let Some(args) = template.args.as_ref() {
                    cmd.args(&args.0);
                }

                if let Some(manifest) = alias.manifest.as_ref() {
                    cmd.arg(format!("--manifest-path={}", manifest));
                }

                cmd.arg("--");

                if let Some(extra_args) = alias.extra_args.as_ref() {
                    cmd.args(&extra_args.0);
                } else if let Some(extra_args) = template.extra_args.as_ref() {
                    cmd.args(&extra_args.0);
                }
            }
            None => {
                if let Some(toolchain) = alias.toolchain.as_ref() {
                    cmd.arg(format!("+{}", toolchain));
                }

                if let Some(subcommand) = alias.subcommand.as_ref() {
                    cmd.arg(subcommand);
                }

                if let Some(args) = alias.args.as_ref() {
                    cmd.args(&args.0);
                }

                if let Some(manifest) = alias.manifest.as_ref() {
                    cmd.arg(format!("--manifest-path={}", manifest));
                }

                cmd.arg("--");

                if let Some(extra_args) = alias.extra_args.as_ref() {
                    cmd.args(&extra_args.0);
                }
            }
        }

        Some(cmd)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_TOML: &str = r#"
        [[template]]
        name = "format"
        extra-args = ["--check"]
        subcommand = "fmt"
        toolchain = "nightly-2024-11-22"

        [[template]]
        name = "lint"
        args = [
            "-Zunstable-options",
            "--all-features",
            "--all-targets",
        ]
        extra-args = [
            "--deny=warnings",
            "--deny=clippy::arithmetic_side_effects",
        ]
        subcommand = "clippy"
        toolchain = "nightly-2024-11-22"

        
        [[alias]]
        name = "fmt-program"
        manifest = "program/Cargo.toml"
        template = "format"

        [[alias]]
        name = "lint-program"
        manifest = "program/Cargo.toml"
        template = "lint"
    "#;

    #[test]
    fn test_parse_toml() {
        let parsed_toml: Toml = toml::de::from_str(TEST_TOML).unwrap();
        assert_eq!(parsed_toml.aliases.len(), 2);
        assert_eq!(parsed_toml.templates.len(), 2);

        let format_template = &parsed_toml.templates.get("format").unwrap();
        assert_eq!(format_template.name, "format");
        assert_eq!(format_template.subcommand, Some("fmt".to_string()));
        assert_eq!(
            format_template.toolchain,
            Some("nightly-2024-11-22".to_string())
        );

        let lint_template = &parsed_toml.templates.get("lint").unwrap();
        assert_eq!(lint_template.name, "lint");
        assert_eq!(
            lint_template.args,
            Some(Args(vec![
                "-Zunstable-options".to_string(),
                "--all-features".to_string(),
                "--all-targets".to_string(),
            ]))
        );
        assert_eq!(
            lint_template.extra_args,
            Some(Args(vec![
                "--deny=warnings".to_string(),
                "--deny=clippy::arithmetic_side_effects".to_string(),
            ]))
        );
        assert_eq!(lint_template.subcommand, Some("clippy".to_string()));
        assert_eq!(
            lint_template.toolchain,
            Some("nightly-2024-11-22".to_string())
        );

        let format_program_alias = &parsed_toml.aliases.get("fmt-program").unwrap();
        assert_eq!(format_program_alias.name, "fmt-program");
        assert_eq!(
            format_program_alias.manifest,
            Some("program/Cargo.toml".to_string())
        );
        assert_eq!(format_program_alias.template, Some("format".to_string()));

        let lint_program_alias = &parsed_toml.aliases.get("lint-program").unwrap();
        assert_eq!(lint_program_alias.name, "lint-program");
        assert_eq!(
            lint_program_alias.manifest,
            Some("program/Cargo.toml".to_string())
        );
        assert_eq!(lint_program_alias.template, Some("lint".to_string()));
    }
}
