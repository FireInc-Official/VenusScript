# ═══════════════════════════════════════════════════════════
# VenusScript v1.5 — Full Feature Showcase (with StdLib)
# ═══════════════════════════════════════════════════════════

from std import *

# ── 1. Using console for output ──
console.print(">>> VENUS-SCRIPT v1.5 SHOWCASE <<<")

# ── 2. Primitives ──
int version = 2
float pi = 3.14
string name = "FireInc"
bool active = true

console.print("Version:")
console.print(version.to_string())

# ── 3. Native String Methods ──
string greeting = "Hello VenusScript"
console.print("String length:")
console.print(greeting.len().to_string())
console.print("Uppercase:")
console.print(greeting.upper())
console.print("Contains 'Venus':")
console.print(greeting.contains("Venus").to_string())

# ── 4. Arrays and Native Array Methods ──
string[] players = ["Max", "Ilya", "Stephen"]
console.print("Players count:")
console.print(players.len().to_string())

players.push("Alex")
console.print("After push:")
console.print(players.len().to_string())

# ── 5. Array Indexing ──
console.print("First player:")
console.print(players[0])

# ── 6. Int Native Methods ──
int negative = -42
console.print("Absolute value:")
console.print(negative.abs().to_string())

# ── 7. Math Module ──
console.print("Math max(10, 20):")
float res = math.max(10.0, 5.5)
console.print("Math Test: 10 + 5.5 = " + res.to_string())

# ── 8. System Utilities ──
console.print("Type of 'name':")
console.print(system.type_of(name))

# ── 9. Control Flow ──
int power = 10 * 5 + 50
if power >= 100
    console.print("System is strong!")
else
    console.print("System is weak.")

# ── 10. Loops ──
console.print("Loading Systems:")
string[] systems = ["Core", "Graphics", "AI"]
for sys in systems
    console.print(sys)

# ── 11. Functions ──
func multiply(int a, int b) -> int
    return a * b

console.print("12 * 4 =")
console.print(multiply(12, 4).to_string())

# ── 12. While Loop ──
int counter = 0
while counter < 3
    console.print("Tick:")
    console.print(counter.to_string())
    counter = counter + 1

# ── 13. Object Constructors ──
object Engine
    int hp = 1000

object Ship
    string model = "Horizon"

# ── 14. Structs and Behaviours ──
struct Vector3
    float x
    float y
    float z

behaviour IFlyable
    func fly()

# ── 15. Assertion ──
system.assert(active == true)
console.print("All assertions passed!")

# ── 16. AST v1.6 Systemic Types ──
# Vectors and Tensors via parentheses
vec3 position(100.5, 50.0, -20.0)
vec4 color(r=1.0, g=0.5, b=0.0, a=1.0)
tensor weights(shape=[4096, 4096], type="float16")

# Memory and Events
buffer VertexData(size=1024)
signal onHit()

# Reference passing
func updatePhysics(ref vec3 targetPos)
    console.print("Physics updated")

# ── 17. Variable Mutation and Object State ──
console.print("--- Mutation Tests ---")

# Full reassignment
vec4 myColor(1.0, 0.0, 0.0, 1.0)
myColor = vec4(0.0, 0.0, 1.0, 1.0)
console.print("Color reassigned to blue:")
console.print(myColor.to_string())

# Dot notation modification
vec3 testPos(0.0, 0.0, 0.0)
testPos.x = 150.5
testPos.y = 10.0
console.print("testPos after dot mutation:")
console.print(testPos.to_string())

# SIMD compound assignment math
vec3 velocity(10.0, 0.0, 5.0)
testPos += velocity
console.print("testPos after += velocity:")
console.print(testPos.to_string())

# Object Method State Mutation
object Car
    float speed = 0.0
    
    func accelerate(float amount)
        speed += amount

Car myCar()
myCar.accelerate(50.0)
console.print("Car speed after accelerate(50.0):")
console.print(myCar.speed.to_string())

console.print(">>> SHOWCASE FINISHED <<<")

