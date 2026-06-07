<div align="center">
  <img src="VenusScript.png" alt="VenusScript Banner" style="border-radius: 20px; max-width: 100%; margin-bottom: 20px;">
  <h1>🌌 VenusScript</h1>
  <p><strong>A Next-Generation, Zero-Cost Compiler Ecosystem by FireInc.</strong></p>
  <img src="https://img.shields.io/badge/version-v0.2.0-blue.svg" alt="Version">
  <img src="https://img.shields.io/badge/status-beta-orange.svg" alt="Status">
</div>

<br>

**VenusScript** is a radically new programming language created by **FireInc**. Designed for absolute minimalism, lightning-fast execution, and architectural elegance, it powers our entire ecosystem. By abandoning the traditional Bytecode VM, VenusScript operates as a highly optimized, single-pass Abstract Syntax Tree (AST) evaluator running on zero-cost Rust closures. 

Everything in VenusScript is built upon a single, universal concept: **Absolute Homoiconicity**. There are no objects or properties in the traditional sense — **everything is a variable**, including control flow (`if`, `while`, `for`) and modules.

## 🚀 Key Features

-   **Zero-Cost Execution:** No Bytecode VM. Execution happens directly on the AST via `Rc<Vec<Node>>` memory arenas, ensuring minimum memory allocation overhead.
-   **Absolute Homoiconicity:** Control structures are just variables. The language is perfectly uniform.
-   **No Equals Sign (`=`):** Assignment and binding are handled purely by the colon (`:`), creating a beautiful, unified declarative aesthetic.
-   **Spatial Harmony:** No curly braces `{}` or semicolons `;`. The language structure is defined entirely by 4-space indentation.
-   **Native Hardware Integration:** Built-in primitives for vectors (`vec2`, `vec3`), matrices, and AI tensors (`tensor`), designed to seamlessly interface with any hardware, including custom SoCs and advanced ecosystems like Horizon Workspace.

## 📦 Installation

Installing VenusScript is just as easy as installing Python or Node.

### Windows
Double-click the `VenusScript_Installer.exe` file, or run it in your terminal:
```bash
.\VenusScript_Installer.exe
```
This will automatically install the compiler and add the `vscript` CLI to your system `PATH`. Restart your terminal and verify the installation:
```bash
vscript --version
```

## 🛠️ Quick Start

Create a file named `hello.vs`:

```venusscript
import std

int count: 10
if(count > 5)
    std.console.print("VenusScript is alive!")
```

Run it via the compiler:
```bash
vscript hello.vs
```

## 📖 Documentation

For a deep dive into the philosophy, Universal 4-Part Anatomy, and the standard library, read the [Language Specification](Language_Spec.md).
