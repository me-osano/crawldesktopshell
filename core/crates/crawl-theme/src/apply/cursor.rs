use crate::config::CursorConfig;
use crate::error::ThemeResult;

/// Apply cursor theme by writing XDG cursor config and environment hints.
/// The user's compositor should be configured to read from ~/.icons/default.
pub fn apply(config: &CursorConfig) -> ThemeResult<()> {
    let icons_dir = dirs::home_dir()
        .ok_or_else(|| crate::error::ThemeError::Other("cannot find home directory".into()))?
        .join(".icons");

    std::fs::create_dir_all(&icons_dir)?;

    let default_dir = icons_dir.join("default");
    std::fs::create_dir_all(&default_dir)?;

    let content = format!(
        r#"[Icon Theme]
Inherits={}
"#,
        config.theme,
    );

    std::fs::write(default_dir.join("index.theme"), content)?;

    Ok(())
}
