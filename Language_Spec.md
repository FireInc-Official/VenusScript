# The Book of VenusScript: Core Architecture & Official Language Specification

**Version:** 0.2.0
**Creators:** FireInc Studio
**Target OS:** Cross-Platform (Windows, Linux, macOS, FlareOS)
**Target Hardware:** Universal (x86_64, ARM, Custom SoCs)

---

## 1. Introduction & Core Philosophy

VenusScript is a radically new programming language engineered by **FireInc**. It was born from a fundamental rejection of the architectural complexity found in traditional Object-Oriented Programming (OOP) and Virtual Machine (VM) ecosystems like Java, Python, or C#.

We asked a simple question: *What if a programming language had only one underlying structural rule?*

### 1.1 "Everything is a Variable" (Absolute Homoiconicity)
In modern languages, a class is different from a function, which is different from a primitive, which is different from an `if` statement. This creates cognitive load and architectural fragmentation.

In VenusScript, **everything is a variable**. 
There are no methods. There are no fields. There are no properties. 
- A number (`int`) is a variable.
- A UI Window (`object`) is just a variable holding other variables.
- A function (`func`) is a variable containing executable variables.
- An `if` statement is a built-in variable type that takes a condition as its argument and execution logic as its content.

By representing both data and control flow under the exact same abstract structure, the compiler becomes incredibly small, blazing fast, and trivial to port directly to any hardware architecture.

### 1.2 Spatial Harmony
VenusScript rejects visual clutter.
- **No Equals Sign (`=`)**: Assignment and data binding use the colon (`:`). The colon naturally implies "this label contains this value".
- **No Braces (`{}`) or Semicolons (`;`)**: The language is strictly indentation-based (4 spaces). Scope is defined visually.
- **Explicit Typing**: The type ALWAYS precedes the identifier. Implicit casting is forbidden.

---

## 2. Compiler Architecture (Internals)

VenusScript is NOT a dynamically interpreted language. It is an **AOT (Ahead-Of-Time) evaluated/compiled language**.

### 2.1 The Core Engine (Rust)
The compiler is written entirely in **Rust** to leverage zero-cost abstractions and the Borrow Checker, guaranteeing memory safety without the performance penalty of a Garbage Collector (GC).

### 2.2 Compilation Pipeline
1. **Lexer (`venus_lexer`)**: A hand-written, stateful scanner that tracks Python-like 4-space indentations and dedentations.
2. **Parser (`venus_parser`)**: Uses a **Pratt Parser** to convert tokens into a highly unified Abstract Syntax Tree (AST). The AST relies almost entirely on a single `VenusVariable` struct.
3. **Semantic Analyzer (`venus_analyzer`)**: Walks the AST to enforce strict typing, prevent implicit conversions, and catch logic errors *before* execution.
4. **Evaluator / Code Generation**: Traverses the AST graph directly in memory. Due to Zero-Copy architectures and `rustc-hash::FxHashMap` lookups, execution overhead is practically non-existent.

### 2.3 The Universal 4-Part Anatomy
Every node in the AST uses the exact same `VenusVariable` struct, containing exactly four properties:
1. **`v_type`**: The declared type (e.g., `int`, `func`, `if`).
2. **`name`**: The identifier (e.g., `playerSpeed`). Sometimes null for literals.
3. **`arguments`**: Passed in parentheses `()`. Used for function parameters or conditional logic.
4. **`content`**: Nested variables. For objects, it's their fields. For functions, it's their executable lines.

### 2.4 Human-Readable Error Engine
Errors are localized to the exact file, line, and column. The console outputs a visual snippet of the code with the error underlined `^^^^` and provides a plain-English explanation (e.g., "You cannot assign a `string` to a `vec3` vector").

### 2.5 Scope-Based Memory Management
Variables are allocated on the stack or heap and are immediately destroyed the moment their execution scope (indentation level) ends. No Garbage Collector micro-stutters.

---

## 3. Syntax & Basics

### 3.1 Variables & Assignment
Types must be explicitly declared. The colon `:` assigns values.
```venusscript
int age: 14
float speed: 1.5
string name: "FireInc"
bool is_active: true
```

Variables can be mutated using the colon again:
```venusscript
age: 15
```

### 3.2 Indentation & Scope
Scope is strictly 4 spaces.
```venusscript
if(age > 10)
    string message: "Older than 10"
    std.console.print(message)
```

### 3.3 Comments
Comments use the hash `#` symbol.
```venusscript
# This is a comment
```

---

## 4. Data Types & Primitives

### 4.1 Base Types
- `int`: 64-bit signed integer.
- `float`: 64-bit floating point.
- `bool`: `true` or `false`.
- `string`: UTF-8 string.

### 4.2 Collections
- `array`: A dynamic list of values.
  ```venusscript
  array numbers: [1, 2, 3]
  ```
- `buffer`: Fixed-size byte memory manipulation for VRAM/low-level management.

### 4.3 Native Primitive Methods
Primitives come with highly-optimized built-in methods.
**Strings:**
- `s.len()`: Returns character count.
- `s.upper()` / `s.lower()`: Casing.
- `s.contains("text")`: Boolean check.
- `s.replace("from", "to")`: String replacement.

**Arrays:**
- `arr.len()`: Returns item count.
- `arr.push(item)`: Appends an item.
- `arr.pop()`: Removes and returns the last item.
- `arr.clear()`: Empties the array.
- `arr.remove(index)`: Removes item at specific integer index.

---

## 5. Hardware & Math Types

Designed for high-performance physics, graphics, and AI, VenusScript includes native SIMD-ready types. They are constructed using function-like syntax.

### 5.1 Vectors
- `vec2`, `vec3`, `vec4`
```venusscript
vec3 position(10.0, 5.0, 0.0)
position.x: 15.5 # Direct, zero-overhead mutation
```

### 5.2 Matrices
- `mat2`, `mat3`, `mat4`

### 5.3 Tensors
- `tensor`: Multi-dimensional arrays natively optimized for AI workloads and matrix multiplication across specialized hardware.

---

## 6. Containers & Structures

### 6.1 `class` (Namespace)
An empty container used purely to group other variables. Contains no OOP logic.
```venusscript
class MathUtils
    func calculate()
        # logic
```

### 6.2 `struct` (Pure Data)
Used exclusively for defining memory-contiguous blocks of raw data. Cannot contain functions. Passed directly to GPU/Stack.
```venusscript
struct Vertex
    vec3 position
    vec2 uv
```

### 6.3 `object` (Logic & State)
State containers and UI components. Objects can contain functions, other variables, and nested objects. They evaluate dynamically.
```venusscript
object Window(width: 1920, height: 1080)
    string title: "FireInc Engine"
    
    func on_draw()
        # draw logic
```

---

## 7. Control Flow (As Variables)

Control flow keywords are simply built-in system types.

### 7.1 `if` / `else`
```venusscript
if(health <= 0)
    std.console.print("Dead")
else
    std.console.print("Alive")
```

### 7.2 `while`
```venusscript
int i: 0
while(i < 10)
    std.console.print(i.to_string())
    i: i + 1
```

### 7.3 `func` & `return`
Functions declare their return type with `->`. `return` is a variable that takes its content via assignment.
```venusscript
func add(int a, int b) -> int
    return: a + b
```

### 7.4 `import`
```venusscript
import std
```

---

## 8. The Standard Library (`std.vs`)

The Standard Library is written in VenusScript but powered by `@native` decorators that bridge calls directly to optimized Rust closures.

### 8.1 `std.math`
- `sin(x)`, `cos(x)`, `tan(x)`, `asin(x)`, `acos(x)`, `atan(x)`
- `pow(base, exp)`, `sqrt(x)`
- `abs(x)`, `min(a, b)`, `max(a, b)`
- `floor(x)`, `ceil(x)`, `round(x)`

### 8.2 `std.system`
- `std.system.time()`: Returns Unix Epoch time in milliseconds (`float`).
- `std.system.sleep(ms)`: Pauses the execution thread.
- `std.system.type_of(obj)`: Returns the type name as a `string`.
- `std.system.assert(condition)`: Halts compilation/execution if false.

### 8.3 `std.file`
- `std.file.read_text(path)`: Returns file contents as a `string`.
- `std.file.write_text(path, content)`: Writes to disk, returns `bool` (success).
- `std.file.exists(path)`: Returns `bool`.

---

## 9. Access Control & Hardware Modifiers

VenusScript includes powerful decorators to explicitly route execution and memory at the compiler level.

### 9.1 Encapsulation
- **`export`**: Exposes a variable/object to other files.
- **`exclude`**: Prevents a variable from being accessed outside its parent module.

### 9.2 Hardware Routing (WIP Architecture)
- **`@native`**: Bridges a VenusScript function signature to a native Rust compiler closure.
- **`@hardware.GPU`**: Routes the execution of a function or structure directly to the GPU shader compiler.
- **`@hardware.NPU`**: Routes execution to Neural Processing Units (like the Horizon Nano accelerator).
- **`@memory.pinned`**: Pins a struct in memory, preventing it from being paged to disk by the host OS.
