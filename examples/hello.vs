import std
# Basic VenusScript Example
int age = 14
string studio = "FireInc"

func calculate(int a, int b) -> int
    return a + b

console.print("--- VenusScript v1.5 Dev Mode ---")
console.print("Age:")
console.print(age)
console.print("Studio:")
console.print(studio)
console.print("Calculation (14 + 5):")
console.print(calculate(age, 5))
