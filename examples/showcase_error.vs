int count = 10
string name = "John"

# Type mismatch during assignment
count = "hello"

# Type mismatch in math operation
vec3 position(0.0, 0.0, 0.0)
vec3 velocity(1.0, 1.0, 1.0)
string text = "speed"

position = position + text

# Type mismatch in variable initialization
vec3 target = "not a vector"

# Valid operations
position = position + velocity
count = count + 5