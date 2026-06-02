from std import *

# 1. Parsing Edge Cases
int a = (10 + 5) * (2 - 1)
float b = 5.0 + 0.5 # Testing .5 float parsing
int[] empty_arr = [1]
string[][] nested_arr = [["a", "b"], ["c"]]

# 2. Scope & Shadowing
int x = 10
if x == 10
    int x = 20
    console.print(x.to_string())

# 3. Trailing commas & weird formatting
vec3 pos(1.0, 2.0, 3.0)

# 4. Math edge cases
int div_zero = 10 / 0
float infinity = 10.0 / 0.0

# 5. Method calls on literals
string len_test = "hello".upper()

# 6. Object nested modification
struct Player
    string name = "Fire"
    vec3 position(0.0, 0.0, 0.0)

Player p1()
p1.position.x = 100.0

# 7. Array out of bounds
string[] arr = ["a"]
console.print(arr[5]) # Should throw runtime error or return void?

# 8. Missing newline at EOF (intentionally leaving no newline)
console.print("End of test")
