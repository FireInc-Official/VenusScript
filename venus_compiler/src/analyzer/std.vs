export object console
    @native
    func print(string msg)

export object math
    @native
    func sin(float x) -> float
    
    @native
    func cos(float x) -> float

    @native
    func tan(float x) -> float

    @native
    func asin(float x) -> float

    @native
    func acos(float x) -> float

    @native
    func atan(float x) -> float

    @native
    func pow(float base, float exp) -> float

    @native
    func sqrt(float x) -> float

    @native
    func abs(float x) -> float

    @native
    func min(float a, float b) -> float

    @native
    func max(float a, float b) -> float

    @native
    func floor(float x) -> float

    @native
    func ceil(float x) -> float

    @native
    func round(float x) -> float

export object system
    @native
    func type_of(object obj) -> string
    
    @native
    func assert(bool condition)

    @native
    func time() -> float

    @native
    func sleep(float ms)

export object file
    @native
    func read_text(string path) -> string

    @native
    func write_text(string path, string content) -> bool

    @native
    func exists(string path) -> bool

export func alloc(int size) -> buffer
