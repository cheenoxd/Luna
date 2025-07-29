print("Performance test - should trigger JIT compilation")

-- Hot loop that should benefit from JIT
local sum = 0
for i = 1, 1000 do
    sum = sum + math.sqrt(i) * math.abs(i - 500)
end

print("Final sum:", sum)
print("This loop should have been JIT compiled!")
