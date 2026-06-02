# ═══════════════════════════════════════════════════════════

export object Class

# ── Built-in Primitives ──
export object string
    func len() -> int
    func upper() -> string
    func contains(string substr) -> bool

export object array
    func len() -> int
    func push(object item)

export object int
    func to_string() -> string
    func abs() -> int

export object float
    func to_string() -> string

export object bool
    func to_string() -> string

export object buffer
    func read_u8(int index) -> int
    func write_u8(int index, int value)
    func len() -> int

export object tensor

export object signal
    func connect(func listener)
    func emit()

export object task
    func resume()

# ── Math Types ──
export struct vec2
    float x
    float y

export struct vec3
    float x
    float y
    float z

export struct vec4
    float x
    float y
    float z
    float w

export func print(string msg)
# VenusScript Standard Library (std)
# Baked into the compiler binary via include_str!
# ═══════════════════════════════════════════════════════════

export object Class

# ── Console I/O ──
export Class console
    func print(string msg)
    func log(string msg)

# ── Math ──
export Class math
    float pi = 3.14159265358979
    float e = 2.71828182845904

    func abs(int val) -> int
    func max(int a, int b) -> int
    func min(int a, int b) -> int

# ── UI (Reserved for future rendering) ──
export Class UI

# ── Hardware (Decorator abstractions) ──
export Class hardware

# ── System Utilities ──
export Class system
    func type_of(object val) -> string
    func assert(bool condition)
