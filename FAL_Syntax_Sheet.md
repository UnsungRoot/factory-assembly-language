# Factory Assembly Language (FAL) - Universal Syntax Cheat Sheet

A complete reference for writing programs in FAL.

---

## 1. Core Mental Model

| Factory Concept | Computer Concept | Syntax Example |
| :--- | :--- | :--- |
| **Workbench** | CPU Register File | Live workbench space |
| **Tray** | CPU Register | `tray1`, `tray2`, ..., `tray14` |
| **Storeroom Bin** | RAM (Memory) | `storeroom[0]`, `storeroom[1]` |
| **Worker** | CPU Core Execution | Executes instructions in order |
| **Workstation** | Function / Subroutine | `workstation "name": ... end` |
| **Supervisor** | OS Kernel / Syscall | `call_supervisor exit`, `say`, `ask` |

---

## 2. Pillar 1: Data Movement & Assignment

```fal
// 1. Assign integer to tray
tray1 = 100
fill 42 into tray2

// 2. Assign character literal (ASCII)
tray3 = '+'
tray4 = 'Z'

// 3. Assign string pointer
tray5 = "Hello, World!"

// 4. Move / copy between trays
move tray1 to tray2

// 5. Clear / zero out a tray
clear tray1
```

---

## 3. Pillar 2: Arithmetic & Comparison

All math operations modify the destination tray in-place.

```fal
// Addition: tray1 = tray1 + tray2 (or number / char)
add tray2 to tray1
add 10 to tray1

// Subtraction: tray1 = tray1 - tray2 (or number / char)
sub tray2 from tray1
sub 5 from tray1

// Multiplication: tray1 = tray1 * tray2 (or number / char)
multiply tray1 by tray2
multiply tray1 by 3

// Division: tray1 = tray1 / tray2 (or number)
divide tray1 by tray2
divide tray1 by 2

// Comparison: compares two values and sets internal CPU flags
compare tray1 with 50
compare tray1 with tray2
compare tray3 with '+'
```

---

## 4. Pillar 3: Branching & Control Flow

### A. Jumps & Conditional Routing
```fal
// Unconditional jump
jump to "target_workstation"

// Conditional jumps based on the last 'compare'
jump_if_equal to "handle_equal"          // je
jump_if_not_equal to "handle_not_equal"  // jne
jump_if_greater to "handle_greater"      // jg
jump_if_less to "handle_less"            // jl
```

### B. Inline One-Line Conditionals
```fal
// Check condition and execute instruction in one line
if tray3 == '+' then call workstation "op_add"
if tray3 == '-' then call workstation "op_sub"
if tray1 > 100 then tray1 = 100
if tray2 == '8' then tray4 = "Matched 8!"
```

---

## 5. Pillar 4: Storeroom (RAM) & Supervisor (I/O)

### A. Storeroom (RAM) Read / Write
```fal
// Save tray to storeroom slot
store tray1 into storeroom[0]
store tray2 into storeroom[1]

// Load from storeroom slot into tray
load storeroom[0] into tray1
load storeroom[1] into tray3
```

### B. Screen Output (`say` / `say_number`)
```fal
// Print text literal
say "Welcome to FAL!"

// Print string from a tray
say tray5

// Print number value from a tray
say_number tray1
```

### C. Interactive User Input (`ask` / `ask_char`)
```fal
// Read a number from user input (waits for user to type and press Enter)
ask tray1
ask_number tray2

// Read a single character from user input (+, -, *, /, etc.)
ask_char tray3
```

### D. Supervisor Commands
```fal
// Exit the program cleanly
call_supervisor exit
```

---

## 6. Workstations (Functions & Subroutines)

Workstations provide isolated scopes and reusable code routines.

```fal
workstation "main":
    say "Starting program..."

    // Call another workstation like a function
    call workstation "calculate_tax"

    say "Finished!"
    call_supervisor exit
end

workstation "calculate_tax":
    tray1 = 1000
    tray2 = 15
    multiply tray1 by tray2
    divide tray1 by 100

    // Return back to caller with result in tray1
    return tray1
end
```

---

## 7. Complete Interactive Calculator Example

```fal
workstation "main":
    say "=== FAL Interactive Calculator ==="
    
    say "Enter first number (A):"
    ask tray1

    say "Enter operator (+, -, *, /):"
    ask_char tray3

    say "Enter second number (B):"
    ask tray2

    // Branch according to the operator:
    if tray3 == '+' then call workstation "op_add"
    if tray3 == '-' then call workstation "op_sub"
    if tray3 == '*' then call workstation "op_mul"
    if tray3 == '/' then call workstation "op_div"

    say "Calculation Result:"
    say_number tray1

    call_supervisor exit
end

workstation "op_add":
    add tray2 to tray1
    return tray1
end

workstation "op_sub":
    sub tray2 from tray1
    return tray1
end

workstation "op_mul":
    multiply tray1 by tray2
    return tray1
end

workstation "op_div":
    divide tray1 by tray2
    return tray1
end
```

---

## 8. CLI Command Quick Reference

```bash
# Run a FAL file
fal run <file.fal>

# Run with the visual factory debugger
fal debug <file.fal>

# Scaffold a new factory project
fal new <project_name>

# Inspect hardware environment & FALZ storage
fal env

# Clean temporary cache and old logs
fal clean
```
