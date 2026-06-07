import sys

with open("eval.rs", "r", encoding="utf-8") as f:
    content = f.read()

# Fix error 1
content = content.replace(
    'imp.items.contains(var.name.as_deref().unwrap_or("<anonymous>"))',
    'imp.items.contains(&var.name.clone().unwrap_or_else(|| "<anonymous>".to_string()))'
)

# Fix error 2
old_eval_val = """        let val = if let Some(expr) = &var.value {
            self.eval_expr(expr)?
        } else if let TypeExpr::Named(ty_name) = &var.type_expr {"""

new_eval_val = """        let mut assigned_val = None;
        if let Some(Node::Variable(anon_var)) = var.content.first() {
            if anon_var.name.is_none() {
                if let Some(Node::ExprStmt(val_expr)) = anon_var.content.first() {
                    assigned_val = Some(self.eval_expr(val_expr)?);
                }
            }
        }

        let val = if let Some(v) = assigned_val {
            v
        } else if let TypeExpr::Named(ty_name) = &var.type_expr {"""

content = content.replace(old_eval_val, new_eval_val)

with open("eval.rs", "w", encoding="utf-8") as f:
    f.write(content)
