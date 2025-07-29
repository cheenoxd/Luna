-- This should trigger JIT compilation due to math operations
print("Testing JIT compilation with math operations")

print("Math operations:")
print(math.abs(-10))
print(math.abs(-20))
print(math.abs(-30))
print(math.abs(-40))
print(math.abs(-50))

print("String operations:")
print(string.upper("test1"))
print(string.upper("test2"))
print(string.upper("test3"))

print("Arithmetic operations:")
print(10 + 5)
print(20 * 2)
print(100 / 4)

print("JIT test complete")

print("Testing JIT compilation with loops")

print("For loop test:")
for i = 1, 5 do
    print("Loop iteration:", i)
    print(math.abs(i - 3))
end

print("JIT test complete")
