use anyhow::Result;
use std::path::{Path, PathBuf};

use super::SkillCategory;

/// Options for scaffolding a new skill.
pub struct ScaffoldOptions {
    /// The name of the skill (kebab-case)
    pub name: String,
    /// A brief description of the skill
    pub description: String,
    /// The category this skill belongs to
    pub category: SkillCategory,
    /// The output directory where the skill will be created
    pub output_dir: PathBuf,
}

/// Generates a complete skill directory structure.
///
/// This function creates a new skill directory with:
/// - `skill.toml` - A manifest file with the provided metadata
/// - `src/lib.rs` - A skeleton `Skill` trait implementation that compiles
/// - `README.md` - Basic documentation with usage instructions
///
/// # Arguments
///
/// * `opts` - The scaffold options containing name, description, category, and output directory
///
/// # Returns
///
/// * `Ok(PathBuf)` - The path to the created skill directory
/// * `Err(anyhow::Error)` - An error if directory creation or file writing fails
///
/// # Example
///
/// ```ignore
/// use aisopod_plugin::skills::{scaffold_skill, ScaffoldOptions, SkillCategory};
/// use std::path::PathBuf;
///
/// let opts = ScaffoldOptions {
///     name: "my-skill".to_string(),
///     description: "A sample skill".to_string(),
///     category: SkillCategory::Productivity,
///     output_dir: PathBuf::from("~/.aisopod/skills"),
/// };
///
/// let skill_dir = scaffold_skill(&opts)?;
/// println!("Created skill at: {:?}", skill_dir);
/// ```
pub fn scaffold_skill(opts: &ScaffoldOptions) -> Result<PathBuf> {
    let skill_dir = opts.output_dir.join(&opts.name);
    
    // Create directory structure
    std::fs::create_dir_all(skill_dir.join("src"))?;
    
    // Generate skill.toml
    let manifest = format!(
        r#"id = "{name}"
name = "{name}"
description = "{description}"
version = "0.1.0"
category = "{category:?}"
required_env_vars = []
required_binaries = []
"#,
        name = opts.name,
        description = opts.description,
        category = opts.category,
    );
    std::fs::write(skill_dir.join("skill.toml"), manifest)?;
    
    // Generate src/lib.rs
    let lib_rs = format!(
        r#"use async_trait::async_trait;
use std::sync::Arc;

use aisopod_plugin::skills::{{Skill, SkillMeta, SkillCategory}};
use aisopod_tools::Tool;

#[derive(Debug)]
pub struct {struct_name} {{
    meta: SkillMeta,
}}

impl {struct_name} {{
    /// Creates a new instance of this skill.
    pub fn new() -> Self {{
        Self {{
            meta: SkillMeta {{
                name: "{name}".to_string(),
                description: "{description}".to_string(),
                version: "0.1.0".to_string(),
                category: SkillCategory::{category:?},
                required_env_vars: vec![],
                required_binaries: vec![],
                platform: None,
            }},
        }}
    }}
}}

impl Default for {struct_name} {{
    fn default() -> Self {{
        Self::new()
    }}
}}

#[async_trait]
impl Skill for {struct_name} {{
    fn id(&self) -> &str {{
        "{name}"
    }}

    fn meta(&self) -> &SkillMeta {{
        &self.meta
    }}

    fn system_prompt_fragment(&self) -> Option<String> {{
        Some("TODO: Describe what this skill provides to the agent.".to_string())
    }}

    fn tools(&self) -> Vec<Arc<dyn Tool>> {{
        vec![]
    }}

    async fn init(&self, _ctx: &aisopod_plugin::skills::SkillContext) -> Result<(), Box<dyn std::error::Error>> {{
        Ok(())
    }}
}}
"#,
        struct_name = to_pascal_case(&opts.name),
        name = opts.name,
        description = opts.description,
        category = opts.category,
    );
    std::fs::write(skill_dir.join("src/lib.rs"), lib_rs)?;
    
    // Generate README.md
    let readme = format!(
        r#"# {name}

{description}

## Category
{category:?}

## Usage
Add `"{name}"` to your agent's `skills` list in the configuration:

```toml
[[agents]]
id = "my-agent"
skills = ["{name}"]
```

## Development
1. Edit `src/lib.rs` to implement your skill's tools and system prompt.
2. Update `skill.toml` with any required environment variables or binaries.
3. Place this directory in `~/.aisopod/skills/` to make it available.

## Structure
```
{name}/
├── src/
│   └── lib.rs      # Main skill implementation
├── skill.toml      # Skill metadata and configuration
└── README.md       # This file
```
"#,
        name = opts.name,
        description = opts.description,
        category = opts.category,
    );
    std::fs::write(skill_dir.join("README.md"), readme)?;
    
    Ok(skill_dir)
}

/// Converts a kebab-case string to PascalCase for struct naming.
///
/// This helper function is used to generate appropriate Rust struct names
/// from skill names that follow kebab-case convention.
///
/// # Arguments
///
/// * `s` - The kebab-case string (e.g., "my-skill")
///
/// # Returns
///
/// * `String` - The PascalCase version (e.g., "MySkill")
///
/// # Example
///
/// ```ignore
/// assert_eq!(to_pascal_case("my-skill"), "MySkill");
/// assert_eq!(to_pascal_case("test-skill-name"), "TestSkillName");
/// assert_eq!(to_pascal_case("a"), "A");
/// ```
pub fn to_pascal_case(s: &str) -> String {
    s.split('-')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => c.to_uppercase().to_string() + &chars.as_str().to_lowercase(),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_to_pascal_case_basic() {
        assert_eq!(to_pascal_case("my-skill"), "MySkill");
        assert_eq!(to_pascal_case("test"), "Test");
        assert_eq!(to_pascal_case("a-b-c"), "ABC");
    }

    #[test]
    fn test_to_pascal_case_empty() {
        assert_eq!(to_pascal_case(""), "");
        assert_eq!(to_pascal_case("-"), "");
    }

    #[test]
    fn test_to_pascal_case_mixed_case() {
        assert_eq!(to_pascal_case("My-Skill"), "MySkill");
        assert_eq!(to_pascal_case("TEST-SKILL"), "TestSkill");
    }

    #[test]
    fn test_scaffold_skill_creates_directory() {
        let dir = tempdir().unwrap();
        let opts = ScaffoldOptions {
            name: "test-skill".to_string(),
            description: "A test skill".to_string(),
            category: SkillCategory::Utility,
            output_dir: dir.path().to_path_buf(),
        };

        let skill_dir = scaffold_skill(&opts).unwrap();
        
        assert!(skill_dir.exists());
        assert!(skill_dir.is_dir());
        assert_eq!(skill_dir.file_name().unwrap(), "test-skill");
    }

    #[test]
    fn test_scaffold_skill_creates_files() {
        let dir = tempdir().unwrap();
        let opts = ScaffoldOptions {
            name: "test-skill".to_string(),
            description: "A test skill".to_string(),
            category: SkillCategory::Productivity,
            output_dir: dir.path().to_path_buf(),
        };

        scaffold_skill(&opts).unwrap();
        
        let skill_dir = dir.path().join("test-skill");
        
        // Check all required files exist
        assert!(skill_dir.join("skill.toml").exists());
        assert!(skill_dir.join("src/lib.rs").exists());
        assert!(skill_dir.join("README.md").exists());
    }

    #[test]
    fn test_scaffold_skill_manifest_content() {
        let dir = tempdir().unwrap();
        let opts = ScaffoldOptions {
            name: "test-skill".to_string(),
            description: "A test skill".to_string(),
            category: SkillCategory::Messaging,
            output_dir: dir.path().to_path_buf(),
        };

        scaffold_skill(&opts).unwrap();
        
        let skill_dir = dir.path().join("test-skill");
        let manifest_content = fs::read_to_string(skill_dir.join("skill.toml")).unwrap();
        
        assert!(manifest_content.contains("id = \"test-skill\""));
        assert!(manifest_content.contains("name = \"test-skill\""));
        assert!(manifest_content.contains("description = \"A test skill\""));
        assert!(manifest_content.contains("version = \"0.1.0\""));
        assert!(manifest_content.contains("category = \"Messaging\""));
    }

    #[test]
    fn test_scaffold_skill_lib_content() {
        let dir = tempdir().unwrap();
        let opts = ScaffoldOptions {
            name: "test-skill".to_string(),
            description: "A test skill".to_string(),
            category: SkillCategory::Utility,
            output_dir: dir.path().to_path_buf(),
        };

        scaffold_skill(&opts).unwrap();
        
        let skill_dir = dir.path().join("test-skill");
        let lib_content = fs::read_to_string(skill_dir.join("src/lib.rs")).unwrap();
        
        // Check that the struct is correctly named (PascalCase)
        assert!(lib_content.contains("pub struct TestSkill"));
        assert!(lib_content.contains("impl TestSkill"));
        assert!(lib_content.contains("fn new() -> Self"));
        
        // Check Skill trait implementation
        assert!(lib_content.contains("impl Skill for TestSkill"));
        assert!(lib_content.contains("fn id(&self) -> &str"));
        assert!(lib_content.contains("fn meta(&self) -> &SkillMeta"));
        assert!(lib_content.contains("fn system_prompt_fragment"));
        assert!(lib_content.contains("fn tools"));
        assert!(lib_content.contains("async fn init"));
    }

    #[test]
    fn test_scaffold_skill_readme_content() {
        let dir = tempdir().unwrap();
        let opts = ScaffoldOptions {
            name: "test-skill".to_string(),
            description: "A test skill".to_string(),
            category: SkillCategory::Integration,
            output_dir: dir.path().to_path_buf(),
        };

        scaffold_skill(&opts).unwrap();
        
        let skill_dir = dir.path().join("test-skill");
        let readme_content = fs::read_to_string(skill_dir.join("README.md")).unwrap();
        
        assert!(readme_content.contains("# test-skill"));
        assert!(readme_content.contains("A test skill"));
        assert!(readme_content.contains("## Category"));
        assert!(readme_content.contains("Integration"));
        assert!(readme_content.contains("## Usage"));
        assert!(readme_content.contains("skills = [\"test-skill\"]"));
    }

    #[test]
    fn test_scaffold_skill_different_categories() {
        let dir = tempdir().unwrap();
        
        for category in [
            SkillCategory::AiMl,
            SkillCategory::System,
        ] {
            let opts = ScaffoldOptions {
                name: format!("test-{:?}", category),
                description: format!("A {:?} skill", category),
                category: category.clone(),
                output_dir: dir.path().to_path_buf(),
            };

            scaffold_skill(&opts).unwrap();
            
            let skill_dir = dir.path().join(format!("test-{:?}", category));
            let lib_content = fs::read_to_string(skill_dir.join("src/lib.rs")).unwrap();
            
            // Verify the category is correctly embedded
            let category_str = match category {
                SkillCategory::AiMl => "AiMl",
                SkillCategory::System => "System",
                _ => unreachable!(),
            };
            assert!(lib_content.contains(&format!("SkillCategory::{}", category_str)));
        }
    }

    #[test]
    fn test_scaffold_skill_creates_src_directory() {
        let dir = tempdir().unwrap();
        let opts = ScaffoldOptions {
            name: "test-skill".to_string(),
            description: "A test skill".to_string(),
            category: SkillCategory::Productivity,
            output_dir: dir.path().to_path_buf(),
        };

        scaffold_skill(&opts).unwrap();
        
        let skill_dir = dir.path().join("test-skill");
        let src_dir = skill_dir.join("src");
        
        assert!(src_dir.exists());
        assert!(src_dir.is_dir());
    }
}
