import * as vscode from 'vscode';
import * as path from 'path';
import * as fs from 'fs';

// --- Documentation dictionary for keywords and standard types ---
const HOVER_DOCS: { [key: string]: string } = {
    'int': '**int**: A 64-bit signed integer.',
    'float': '**float**: A 64-bit floating point number.',
    'string': '**string**: A UTF-8 text string.',
    'bool': '**bool**: A boolean value (`true` or `false`).',
    'vec2': '**vec2**: A 2D spatial vector (x, y).',
    'vec3': '**vec3**: A 3D spatial vector (x, y, z).',
    'vec4': '**vec4**: A 4D spatial vector (x, y, z, w).',
    'mat2': '**mat2**: A 2x2 transformation matrix.',
    'mat3': '**mat3**: A 3x3 transformation matrix.',
    'mat4': '**mat4**: A 4x4 transformation matrix.',
    'tensor': '**tensor**: A multi-dimensional array for mathematical computations.',
    'buffer': '**buffer**: A raw binary data buffer.',
    'object': '**object**: The base structure type in VenusScript. Acts as a blueprint for variables.',
    'struct': '**struct**: An alias for `object`.',
    'class': '**class**: An alias for `object`. Used for Object-Oriented concepts.',
    'func': '**func**: Defines a function or method.',
    'import': '**import**: Includes a module or standard library (e.g., `import std`).',
    'export': '**export**: Makes a variable or structure accessible from outside the file.',
    'return': '**return**: Exits a function and optionally returns a value.',
    'if': '**if**: Conditional execution block.',
    'elif': '**elif**: Secondary conditional block ("else if").',
    'else': '**else**: Fallback block for an `if` chain.',
    'for': '**for**: Loop over an iterable array or range.',
    'while': '**while**: Execute a block as long as the condition is true.',
};

export function activate(context: vscode.ExtensionContext) {
    console.log('VenusScript extension is now active!');

    // --- Command: Run File ---
    let runDisposable = vscode.commands.registerCommand('venusscript.runFile', () => {
        const editor = vscode.window.activeTextEditor;
        if (!editor) {
            vscode.window.showErrorMessage('No active file to run.');
            return;
        }

        const document = editor.document;
        if (document.languageId !== 'venusscript') {
            vscode.window.showErrorMessage('Active file is not a VenusScript file (.vs).');
            return;
        }

        const filePath = document.fileName;
        
        document.save().then(() => {
            let terminal = vscode.window.terminals.find(t => t.name === 'VenusScript');
            if (!terminal) {
                terminal = vscode.window.createTerminal('VenusScript');
            }
            terminal.show();

            // Use the global vscript command
            const command = `vscript "${filePath}"`;
            
            terminal.sendText(command);
        });
    });

    // --- Hover Provider (IntelliSense) ---
    let hoverDisposable = vscode.languages.registerHoverProvider('venusscript', {
        provideHover(document, position, token) {
            const range = document.getWordRangeAtPosition(position);
            if (!range) return null;

            const word = document.getText(range);

            // 1. Check if it's a known keyword/type
            if (HOVER_DOCS[word]) {
                return new vscode.Hover(new vscode.MarkdownString(HOVER_DOCS[word]));
            }

            // 2. Backward search for variable declaration
            // e.g. "int age", "string name", "object myObj"
            for (let i = position.line; i >= 0; i--) {
                const lineText = document.lineAt(i).text;
                // Regex looks for: <type> <word> = ... OR <type> <word>
                const regex = new RegExp(`\\b([a-zA-Z0-9_]+)\\s+${word}\\b`);
                const match = lineText.match(regex);
                if (match) {
                    const typeName = match[1];
                    // Make sure it's not a keyword catching another keyword
                    if (!['return', 'if', 'elif', 'else', 'while', 'for', 'import', 'export'].includes(typeName)) {
                        const hoverText = new vscode.MarkdownString();
                        hoverText.appendCodeblock(`(variable) ${word}: ${typeName}`, 'venusscript');
                        return new vscode.Hover(hoverText);
                    }
                }
            }

            return null;
        }
    });

    // --- Definition Provider (Go To Definition) ---
    let defDisposable = vscode.languages.registerDefinitionProvider('venusscript', {
        provideDefinition(document, position, token) {
            const range = document.getWordRangeAtPosition(position);
            if (!range) return null;

            const word = document.getText(range);
            const lineText = document.lineAt(position.line).text;

            // 1. Check if it's an import statement
            if (lineText.trim().startsWith('import ') || lineText.trim().startsWith('from ')) {
                // If it's std, map it to the compiler's std.vs
                if (word === 'std') {
                    const stdPath = 'D:\\FireInc Projects and Workspace\\VenusScript\\venus_compiler\\src\\analyzer\\std.vs';
                    if (fs.existsSync(stdPath)) {
                        return new vscode.Location(vscode.Uri.file(stdPath), new vscode.Position(0, 0));
                    }
                } else {
                    // Try local file resolution (e.g. import my_module -> my_module.vs)
                    const dir = path.dirname(document.fileName);
                    const modulePath = path.join(dir, `${word}.vs`);
                    if (fs.existsSync(modulePath)) {
                        return new vscode.Location(vscode.Uri.file(modulePath), new vscode.Position(0, 0));
                    }
                }
            }

            // 2. Search backward for the definition of the variable/function
            for (let i = position.line; i >= 0; i--) {
                const searchLine = document.lineAt(i).text;
                
                // Match "<type> word" OR "func word" OR "object class word"
                const varRegex = new RegExp(`\\b[a-zA-Z0-9_]+\\s+${word}\\b`);
                const funcRegex = new RegExp(`\\bfunc\\s+${word}\\b`);
                const classRegex = new RegExp(`\\bclass\\s+${word}\\b`);

                if (varRegex.test(searchLine) || funcRegex.test(searchLine) || classRegex.test(searchLine)) {
                    const matchIndex = searchLine.indexOf(word);
                    // Don't jump if we are already on the definition line itself, 
                    // unless we just want to highlight it.
                    return new vscode.Location(
                        document.uri,
                        new vscode.Position(i, matchIndex)
                    );
                }
            }

            return null;
        }
    });

    context.subscriptions.push(runDisposable, hoverDisposable, defDisposable);
}

export function deactivate() {}
