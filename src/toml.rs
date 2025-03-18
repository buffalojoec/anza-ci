//! Anza CI configuration TOML file.

use {
    crate::parser::{map_of_keyed_elements, TomlNamedElement},
    serde::Deserialize,
    std::{collections::HashMap, process::Command},
};

// `cargo` is the default command.
const DEFAULT_COMMAND: &str = "cargo";

#[derive(Clone, Debug, Default, PartialEq, Deserialize)]
struct Args(Vec<String>);

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct Template {
    name: String,
    args: Option<Args>,
    command: Option<String>,
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
    command: Option<String>,
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

        match alias.template.as_ref().and_then(|t| self.templates.get(t)) {
            Some(template) => {
                // Alias takes precedence over template.

                let mut cmd = if let Some(command) = alias.command.as_ref() {
                    Command::new(command)
                } else if let Some(command) = template.command.as_ref() {
                    Command::new(command)
                } else {
                    Command::new(DEFAULT_COMMAND)
                };

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

                Some(cmd)
            }
            None => {
                let mut cmd = if let Some(command) = alias.command.as_ref() {
                    Command::new(command)
                } else {
                    Command::new(DEFAULT_COMMAND)
                };

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

                Some(cmd)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const TEST_TOML: &str = r#"
        [[template]]
        name = "format:rust"
        command = "cargo"
        extra-args = ["--check"]
        subcommand = "fmt"
        toolchain = "nightly-2024-11-22"

        [[template]]
        name = "format:js"
        command = "pnpm"
        subcommand = "prettier"

        [[template]]
        name = "lint:rust"
        args = [
            "-Zunstable-options",
            "--all-features",
            "--all-targets",
        ]
        command = "cargo"
        extra-args = [
            "--deny=warnings",
            "--deny=clippy::arithmetic_side_effects",
        ]
        subcommand = "clippy"
        toolchain = "nightly-2024-11-22"

        
        [[alias]]
        name = "fmt-program"
        manifest = "program/Cargo.toml"
        template = "format:rust"
        
        [[alias]]
        name = "fmt-client-js"
        manifest = "clients/js/package.json"
        template = "format:js"

        [[alias]]
        name = "lint-program"
        manifest = "program/Cargo.toml"
        template = "lint:rust"
    "#;

    #[test]
    fn test_parse_toml() {
        let parsed_toml: Toml = toml::de::from_str(TEST_TOML).unwrap();
        assert_eq!(parsed_toml.aliases.len(), 3);
        assert_eq!(parsed_toml.templates.len(), 3);

        let format_rust_template = &parsed_toml.templates.get("format:rust").unwrap();
        assert_eq!(format_rust_template.name, "format:rust");
        assert_eq!(format_rust_template.args, None);
        assert_eq!(format_rust_template.command, Some("cargo".to_string()));
        assert_eq!(
            format_rust_template.extra_args,
            Some(Args(vec!["--check".to_string()]))
        );
        assert_eq!(format_rust_template.subcommand, Some("fmt".to_string()));
        assert_eq!(
            format_rust_template.toolchain,
            Some("nightly-2024-11-22".to_string())
        );

        let format_js_template = &parsed_toml.templates.get("format:js").unwrap();
        assert_eq!(format_js_template.name, "format:js");
        assert_eq!(format_js_template.args, None);
        assert_eq!(format_js_template.command, Some("pnpm".to_string()));
        assert_eq!(format_js_template.extra_args, None);
        assert_eq!(format_js_template.subcommand, Some("prettier".to_string()));
        assert_eq!(format_js_template.toolchain, None);

        let lint_rust_template = &parsed_toml.templates.get("lint:rust").unwrap();
        assert_eq!(lint_rust_template.name, "lint:rust");
        assert_eq!(
            lint_rust_template.args,
            Some(Args(vec![
                "-Zunstable-options".to_string(),
                "--all-features".to_string(),
                "--all-targets".to_string(),
            ]))
        );
        assert_eq!(lint_rust_template.command, Some("cargo".to_string()));
        assert_eq!(
            lint_rust_template.extra_args,
            Some(Args(vec![
                "--deny=warnings".to_string(),
                "--deny=clippy::arithmetic_side_effects".to_string(),
            ]))
        );
        assert_eq!(lint_rust_template.subcommand, Some("clippy".to_string()));
        assert_eq!(
            lint_rust_template.toolchain,
            Some("nightly-2024-11-22".to_string())
        );

        let format_program_alias = &parsed_toml.aliases.get("fmt-program").unwrap();
        assert_eq!(format_program_alias.name, "fmt-program");
        assert_eq!(
            format_program_alias.manifest,
            Some("program/Cargo.toml".to_string())
        );
        assert_eq!(
            format_program_alias.template,
            Some("format:rust".to_string())
        );

        let format_client_js_alias = &parsed_toml.aliases.get("fmt-client-js").unwrap();
        assert_eq!(format_client_js_alias.name, "fmt-client-js");
        assert_eq!(
            format_client_js_alias.manifest,
            Some("clients/js/package.json".to_string())
        );
        assert_eq!(
            format_client_js_alias.template,
            Some("format:js".to_string())
        );

        let lint_program_alias = &parsed_toml.aliases.get("lint-program").unwrap();
        assert_eq!(lint_program_alias.name, "lint-program");
        assert_eq!(
            lint_program_alias.manifest,
            Some("program/Cargo.toml".to_string())
        );
        assert_eq!(lint_program_alias.template, Some("lint:rust".to_string()));
    }
}
