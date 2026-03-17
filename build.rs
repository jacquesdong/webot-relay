use std::collections::HashMap;
use std::env;
use std::fs;
use std::process::Command;

fn main() {
    // Generate i18n module
    if let Err(e) = generate_i18n_module() {
        eprintln!("Error generating i18n module: {}", e);
        std::process::exit(1);
    }

    // Generate build version
    generate_build_version();
}

fn generate_build_version() {
    let mut build_version = env!("CARGO_PKG_VERSION").to_string();

    if let Some(date) = git_commit_date() {
        build_version.push('-');
        build_version.push_str(&date);
    }

    if let Some(hash) = git_commit_hash() {
        build_version.push('-');
        build_version.push_str(&hash);
    }

    let dirty = is_git_dirty();
    if dirty {
        build_version.push('+');
    }

    println!("cargo:warning=Build version: {}", build_version);
    println!("cargo:rustc-env=BUILD_VERSION={}", build_version);

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=.git/HEAD");
    println!("cargo:rerun-if-changed=.git/index");
    println!("cargo:rerun-if-changed=locales");
}

fn generate_i18n_module() -> Result<(), Box<dyn std::error::Error>> {
    generate_i18n_module_from_file("locales/app.yml", "src/i18n.rs")
}

fn generate_i18n_module_from_file(
    locales_path: &str,
    dest_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Read locales file
    let locales_content = fs::read_to_string(locales_path)?;

    // Parse YAML
    let yaml: serde_yaml::Value = serde_yaml::from_str(&locales_content)?;

    // Extract translation entries
    let mut entries = Vec::new();
    extract_translations(&yaml, "", &mut entries);

    // Generate i18n code
    let code = generate_i18n_code(&entries, locales_path);

    // Write to file
    let out_dir = env::var("CARGO_MANIFEST_DIR")?;
    let dest_full_path = std::path::Path::new(&out_dir).join(dest_path);
    fs::write(&dest_full_path, code)?;

    println!(
        "cargo:warning=Generated i18n module with {} translations",
        entries.len()
    );
    Ok(())
}

struct TranslationEntry {
    key: String,
    module_path: String,
    function_name: String,
    params: Vec<String>,
    en_text: String,
    zh_text: String,
}

fn extract_translations(
    yaml: &serde_yaml::Value,
    prefix: &str,
    entries: &mut Vec<TranslationEntry>,
) {
    if let serde_yaml::Value::Mapping(map) = yaml {
        for (key, value) in map {
            if let serde_yaml::Value::String(key_str) = key {
                if key_str.starts_with('_') {
                    continue; // Skip special keys like _version
                }

                let full_key = if prefix.is_empty() {
                    key_str.clone()
                } else {
                    format!("{}.{}", prefix, key_str)
                };

                if let serde_yaml::Value::Mapping(inner_map) = value {
                    if inner_map.contains_key(&serde_yaml::Value::String("en".to_string())) {
                        // This is a translation entry
                        let en_text = inner_map
                            .get(&serde_yaml::Value::String("en".to_string()))
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();

                        let zh_text = inner_map
                            .get(&serde_yaml::Value::String("zh-CN".to_string()))
                            .and_then(|v| v.as_str())
                            .unwrap_or("")
                            .to_string();

                        // Extract parameters
                        let params = extract_parameters(&en_text);

                        // Determine module path and function name
                        let parts: Vec<&str> = full_key.split('.').collect();
                        let module_path = parts[0..parts.len() - 1].join(".");
                        let function_name = parts.last().unwrap().to_string();

                        entries.push(TranslationEntry {
                            key: full_key,
                            module_path,
                            function_name,
                            params,
                            en_text,
                            zh_text,
                        });
                    } else {
                        // Recurse into nested map
                        extract_translations(value, &full_key, entries);
                    }
                }
            }
        }
    }
}

fn extract_parameters(text: &str) -> Vec<String> {
    let mut params = Vec::new();
    let mut chars = text.chars();

    while let Some(c) = chars.next() {
        if c == '%' && chars.as_str().starts_with('{') {
            chars.next(); // Consume '{'
            let mut param = String::new();

            while let Some(c) = chars.next() {
                if c == '}' {
                    break;
                }
                param.push(c);
            }

            if !param.is_empty() {
                params.push(param);
            }
        }
    }

    params
}

struct ModuleNode {
    name: String,
    functions: Vec<TranslationEntry>,
    children: HashMap<String, ModuleNode>,
}

impl ModuleNode {
    fn new(name: String) -> Self {
        ModuleNode {
            name,
            functions: Vec::new(),
            children: HashMap::new(),
        }
    }
}

fn build_module_tree(entries: &[TranslationEntry]) -> ModuleNode {
    let mut root = ModuleNode::new("".to_string());

    for entry in entries {
        if entry.module_path.is_empty() {
            // Root level function
            root.functions.push(TranslationEntry {
                key: entry.key.clone(),
                module_path: entry.module_path.clone(),
                function_name: entry.function_name.clone(),
                params: entry.params.clone(),
                en_text: entry.en_text.clone(),
                zh_text: entry.zh_text.clone(),
            });
        } else {
            let parts: Vec<&str> = entry.module_path.split('.').collect();
            let mut current = &mut root;

            for part in parts {
                current = current
                    .children
                    .entry(part.to_string())
                    .or_insert_with(|| ModuleNode::new(part.to_string()));
            }

            current.functions.push(TranslationEntry {
                key: entry.key.clone(),
                module_path: entry.module_path.clone(),
                function_name: entry.function_name.clone(),
                params: entry.params.clone(),
                en_text: entry.en_text.clone(),
                zh_text: entry.zh_text.clone(),
            });
        }
    }

    root
}

fn generate_i18n_code(entries: &[TranslationEntry], locales_path: &str) -> String {
    let mut code = String::new();

    // File header
    code.push_str("// Auto-generated by build.rs - DO NOT EDIT\n");
    code.push_str(&format!(
        "// This file is generated from {}\n\n",
        locales_path
    ));

    // Build module tree
    let root = build_module_tree(entries);

    // Generate code from module tree
    for child in root.children.values() {
        code.push_str(&generate_module_from_node(child, ""));
    }

    // Generate root level functions
    for func in &root.functions {
        code.push_str(&generate_function(func, ""));
    }

    code
}

fn generate_module_from_node(node: &ModuleNode, indent: &str) -> String {
    let mut code = String::new();

    code.push_str(&format!("{}pub mod {} {{\n", indent, node.name));

    let child_indent = format!("{}    ", indent);

    // Import t! macro only if this module has functions
    if !node.functions.is_empty() {
        code.push_str(&format!("{}    use rust_i18n::t;\n\n", indent));
    }

    // Generate functions
    for func in &node.functions {
        code.push_str(&generate_function(func, &child_indent));
    }

    // Generate children
    for child in node.children.values() {
        code.push_str(&generate_module_from_node(child, &child_indent));
    }

    code.push_str(&format!("{}}}\n\n", indent));

    code
}

fn generate_function(entry: &TranslationEntry, indent: &str) -> String {
    let mut code = String::new();

    // Function documentation
    code.push_str(&format!(
        "{}/// Get translation for \"{}\"\n",
        indent, entry.key
    ));
    code.push_str(&format!("{}/// \n", indent));
    code.push_str(&format!("{}/// English: \"{}\"\n", indent, entry.en_text));
    code.push_str(&format!("{}/// Chinese: \"{}\"\n", indent, entry.zh_text));

    if !entry.params.is_empty() {
        code.push_str(&format!("{}/// \n", indent));
        code.push_str(&format!("{}/// # Parameters\n", indent));
        for param in &entry.params {
            code.push_str(&format!("{}/// - `{}`: Parameter\n", indent, param));
        }
    }

    // Function signature
    if entry.params.is_empty() {
        // No parameters
        code.push_str(&format!(
            "{}pub fn {}() -> String {{\n",
            indent, entry.function_name
        ));
        code.push_str(&format!(
            "{}    t!(\"{}\").to_string()\n",
            indent, entry.key
        ));
        code.push_str(&format!("{}}}\n\n", indent));
    } else {
        // With parameters
        let params: Vec<String> = entry
            .params
            .iter()
            .map(|p| format!("{}: &str", p))
            .collect();
        let args: Vec<String> = entry
            .params
            .iter()
            .map(|p| format!("{} = {}", p, p))
            .collect();

        code.push_str(&format!(
            "{}pub fn {}({}) -> String {{\n",
            indent,
            entry.function_name,
            params.join(", ")
        ));
        code.push_str(&format!(
            "{}    t!(\"{}\", {}).to_string()\n",
            indent,
            entry.key,
            args.join(", ")
        ));
        code.push_str(&format!("{}}}\n\n", indent));
    }

    code
}

fn git_commit_date() -> Option<String> {
    let output = Command::new("git")
        .args(["log", "-1", "--format=%cd", "--date=format:%Y%m%d"])
        .output();

    match output {
        Ok(output) if output.status.success() => {
            Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
        }
        _ => {
            println!("cargo:warning=failed to get commit date");
            None
        }
    }
}

fn git_commit_hash() -> Option<String> {
    let output = Command::new("git")
        .args(["rev-parse", "--short=8", "HEAD"])
        .output();

    match output {
        Ok(output) if output.status.success() => {
            Some(String::from_utf8_lossy(&output.stdout).trim().to_string())
        }
        _ => {
            println!("cargo:warning=failed to get commit hash");
            None
        }
    }
}

#[rustfmt::skip]
fn is_git_dirty() -> bool {
    let output = Command::new("git")
        .args(["status", "--porcelain"])
        .output();

    match output {
        Ok(output) if output.status.success() => !output.stdout.is_empty(),
        _ => false,
    }
}
