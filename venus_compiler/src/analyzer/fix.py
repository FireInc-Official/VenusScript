import sys

with open("eval.rs", "r", encoding="utf-8") as f:
    content = f.read()

content = content.replace(
    "var.name.clone()", 
    "var.name.clone().unwrap_or_else(|| \"<anonymous>\".to_string())"
)
content = content.replace(
    "&var.name",
    "var.name.as_deref().unwrap_or(\"<anonymous>\")"
)

with open("eval.rs", "w", encoding="utf-8") as f:
    f.write(content)
