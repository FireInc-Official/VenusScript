from std import *

# ── Buffer ──
print("Testing buffer...")
buffer b(size=10)
b.write_u8(0, 42)
int val = b.read_u8(0)
print("Buffer size: " + b.len().to_string())
print("Buffer[0]: " + val.to_string())

# ── Tensor ──
print("Testing tensor...")
tensor t(shape=[2, 2], data=[1.0, 0.0, 0.0, 1.0])
print("Tensor created!")

# ── Signal ──
print("Testing signal...")
signal on_click()

func my_listener()
    print("Signal emitted!")

on_click.connect(my_listener)
on_click.emit()

# ── Task ──
print("Testing task...")
task t1()
    print("Hello from task!")

t1.resume()

# ── Enum ──
print("Testing enum...")
enum ProcessState
    Idle
    Running
    Terminated

print("ProcessState.Running: " + ProcessState.Running.to_string())
