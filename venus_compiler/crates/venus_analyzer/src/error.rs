use venus_lexer::token::Span;

pub struct VenusError {
    pub title: String,
    pub message: String,
    pub hint: Option<String>,
    pub span: Span,
}

pub struct VenusErrorHandler<'a> {
    source_code: &'a str,
    file_name: &'a str,
}

impl<'a> VenusErrorHandler<'a> {
    pub fn new(source_code: &'a str, file_name: &'a str) -> Self {
        Self { source_code, file_name }
    }

    pub fn report(&self, err: &VenusError) {
        println!("\n\x1b[1;31m[{}]\x1b[0m", err.title); // Bold Red
        println!("\x1b[1mLocation:\x1b[0m {}:{}", self.file_name, err.span.line);
        println!("");

        // Extract the specific line from source code
        let lines: Vec<&str> = self.source_code.lines().collect();
        let line_index = (err.span.line.saturating_sub(1)) as usize;
        
        if line_index < lines.len() {
            let line_text = lines[line_index];
            println!("{} | {}", err.span.line, line_text);
            
            // Calculate padding for the squiggly line
            let prefix_len = err.span.line.to_string().len() + 3; // "14 | "
            let padding = " ".repeat(prefix_len + (err.span.column.saturating_sub(1) as usize));
            let squiggly_len = err.span.end.saturating_sub(err.span.start).max(1);
            let squiggles = "^".repeat(squiggly_len);
            
            println!("{}\x1b[1;31m{}\x1b[0m", padding, squiggles);
        }

        println!("");
        println!("\x1b[1mExplanation:\x1b[0m {}", err.message);
        
        if let Some(hint) = &err.hint {
            println!("💡 \x1b[1;33mHint:\x1b[0m {}", hint);
        }
        println!("");
    }
}
