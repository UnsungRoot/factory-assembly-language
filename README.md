# Factory Assembly Language (FAL)

> Low-level programming made simple through a factory mental model.
> Written entirely in Rust by Kasish. Open-source under MIT License.

[![CI](https://github.com/UnsungRoot/factory-assembly-language/actions/workflows/ci.yml/badge.svg)](https://github.com/UnsungRoot/factory-assembly-language/actions)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Language: Rust](https://img.shields.io/badge/Built%20with-Rust-orange.svg)](https://www.rust-lang.org/)
[![Platform: Linux | macOS](https://img.shields.io/badge/Platform-Linux%20%7C%20macOS-green.svg)](#installation)

---

## What is FAL?

FAL (Factory Assembly Language) is a low-level programming language with a native JIT compiler
that makes assembly-level thinking accessible to everyone, from beginners to seasoned engineers.

Instead of cryptic register names like RAX, RBX, and complex syscall conventions, FAL uses
a real-world factory analogy that maps directly to how a CPU actually works.

You write .fal files. FAL detects your CPU and OS at runtime, maps virtual names to physical
hardware registers, JIT-compiles to native machine code, and runs it directly on your silicon.
No virtual machine. No interpreter. Pure native speed.

---

## The Factory Mental Model

Forget registers, syscalls, and stack frames. Imagine this:

```
+----------------------------------------------------------------------+
|                        THE FACTORY FLOOR                             |
|                                                                      |
|  +-----------------+   +--------+   +----------------------------+  |
|  | WORKBENCH       |   | BUTTON |   | STOREROOM (RAM)            |  |
|  | [tray1] [tray2] |   |        |   | [bin 0] [bin 1] [bin 2]... |  |
|  | [tray3] [tray4] |   | Call   |   |                            |  |
|  | ...up to tray14 |   | Supvsr |   | Unlimited numbered bins.   |  |
|  |                 |   |        |   | Slower to reach but stores |  |
|  | Instant access. |   |        |   | anything while you work.   |  |
|  | Limited space.  |   |        |   |                            |  |
|  +-----------------+   +--------+   +----------------------------+  |
|                                                                      |
|  WORKER (CPU):     Reads instructions one at a time.                |
|  WORKSTATION:      A named room with its own set of private trays.  |
+----------------------------------------------------------------------+
```

| Factory Term    | Computer Term       | FAL Keyword           |
|-----------------|---------------------|-----------------------|
| Tray            | CPU Register        | tray1, tray2, ...     |
| Storeroom Bin   | RAM (Stack Memory)  | storeroom[0]          |
| Supervisor      | OS Syscall          | call_supervisor       |
| Workstation     | Function/Subroutine | workstation "name":   |
| Worker          | CPU Core            | (the JIT engine)      |

---

## The Four Pillars of FAL

Every program ever written comes down to exactly four operations:

### Pillar 1: Filling and Moving Data Between Trays
```
tray1 = 42
tray2 = 'A'
tray3 = "Hello, World!"
move tray1 to tray2
clear tray1
```

### Pillar 2: Arithmetic and Comparison
```
add tray2 to tray1
sub 10 from tray1
multiply tray1 by 3
divide tray1 by 2
compare tray1 with 100
```

### Pillar 3: Control Flow and Routing
```
jump_if_equal to "workstation_name"
jump_if_not_equal to "other_workstation"
jump_if_greater to "big_handler"
jump_if_less to "small_handler"

// Inline one-liners:
if tray2 == '8' then tray4 = "Matched!"
if tray1 > 100 then tray1 = 100
```

### Pillar 4: Storeroom (RAM) and Supervisor (OS)
```
store tray1 into storeroom[0]
load storeroom[0] into tray2

say "Hello from FAL!"
say tray3

call_supervisor exit
```

---

## Workstations (Functions)

Each workstation is a named, isolated room with its own private virtual trays.
This is what makes FAL dramatically more readable than raw assembly.

```
workstation "main":
    tray1 = 40
    tray2 = 2
    add tray2 to tray1

    call workstation "print_result"
    call_supervisor exit
end

workstation "print_result":
    say "Calculation complete!"
    return tray1
end
```

---

## Smart Shuffler Layer (SSL)

FAL's SSL bridges readable virtual trays and physical silicon registers.

- You write tray1, tray2, ..., tray14.
- FAL detects your exact CPU architecture and OS supervisor at runtime on every execution.
- SSL maps them to the correct physical registers (RAX, RBX... on x86_64 or X0, X1... on ARM64).
- Registers clobbered by syscalls are automatically preserved and restored.

You never need to care about register conventions again.

---

## FALZ: Global Environment Storage

FALZ is FAL's global storage and environment system, installed at ~/.falz/.

```
~/.falz/
  bin/          ->  The FAL executable itself
  env.json      ->  Hardware profile (CPU arch, OS, SSL register pool)
  cache/        ->  Compiled artifact cache
  storeroom/    ->  Persistent global storeroom slots
  logs/         ->  Execution trace logs per run
```

Every time a .fal file runs, FALZ re-detects your CPU and OS in real time and updates env.json.
FAL always runs correctly even if you move its binary to a different machine or architecture.

---

## Installation

### Linux and macOS (One-Line Installer)
```bash
curl -sSf https://raw.githubusercontent.com/UnsungRoot/factory-assembly-language/main/install.sh | sh
```

After installation, reload your shell:
```bash
source ~/.bashrc   # or: source ~/.zshrc
```

Verify the installation:
```bash
fal env
```

### Build from Source (Requires Rust)
```bash
# 1. Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 2. Clone FAL
git clone https://github.com/UnsungRoot/factory-assembly-language.git
cd fal

# 3. Install
./install.sh
```

---

## Dependencies

FAL is designed to be as lean as possible.

### Runtime Dependencies
| Dependency | Purpose                                               |
|------------|-------------------------------------------------------|
| None       | FAL programs compile to self-contained native binaries. No runtime libraries are needed to run .fal files. |

### Build-Time Dependencies (Rust Crates)
| Crate     | Version | Purpose                                                              |
|-----------|---------|----------------------------------------------------------------------|
| memmap2   | 0.9     | Allocates anonymous read-write-execute memory pages so the JIT engine can write and run native machine code at runtime. |

### System Requirements
| Requirement | Details                                                         |
|-------------|-----------------------------------------------------------------|
| OS          | Linux (x86_64 or aarch64), macOS (x86_64 or Apple Silicon)     |
| CPU         | Any x86_64 or ARM64 processor                                   |
| Rust        | 1.70 or newer (for building from source only)                   |
| Disk        | ~2 MB for the FAL binary and FALZ storage                       |
| RAM         | Minimal. FAL programs use their own 4 KB stack allocation at runtime. |

---

## CLI Reference

```
fal run <file.fal>      Run a .fal program with native JIT compilation
fal debug <file.fal>    Run with the visual debugger (code size, workstations, register pool)
fal new <project_name>  Scaffold a new FAL project with a starter factory.fal
fal env                 Show hardware environment report (CPU, OS, SSL register pool, FALZ)
fal doctor              Alias for fal env
fal clean               Clean FALZ cache and session logs
fal help                Show help and command reference
```

---

## Complete Working Example

```
workstation "main":
    say "=== Welcome to Factory Assembly Language (FAL) ==="

    tray2 = '8'
    tray4 = "Default value"

    if tray2 == '8' then tray4 = "Hello, World! Tray matched '8' perfectly."

    say tray4

    call workstation "calculate_bonus"
    say "Returned to main workstation!"

    call_supervisor exit
end

workstation "calculate_bonus":
    tray1 = 100
    tray2 = 50
    add tray2 to tray1
    store tray1 into storeroom[0]
    return tray1
end
```

Output:
```
=== Welcome to Factory Assembly Language (FAL) ===
Hello, World! Tray matched '8' perfectly.
Returned to main workstation!
```

---

## Full Language Reference

### Assignment
| Syntax                 | Description                           |
|------------------------|---------------------------------------|
| tray1 = 42             | Assign integer to tray                |
| tray2 = 'A'            | Assign character to tray              |
| tray3 = "Hello!"       | Assign string pointer to tray         |
| move tray1 to tray2    | Copy tray1 value into tray2           |
| clear tray1            | Zero out tray1                        |
| fill 99 into tray1     | Alternative integer assignment        |

### Arithmetic
| Syntax                  | Description                           |
|-------------------------|---------------------------------------|
| add tray2 to tray1      | tray1 = tray1 + tray2                |
| add 10 to tray1         | tray1 = tray1 + 10                   |
| sub 5 from tray1        | tray1 = tray1 - 5                    |
| multiply tray1 by 3     | tray1 = tray1 * 3                    |
| divide tray1 by 2       | tray1 = tray1 / 2                    |

### Comparison and Branching
| Syntax                          | Description                        |
|---------------------------------|------------------------------------|
| compare tray1 with 100          | Sets CPU flags for next jump       |
| jump to "name"                  | Unconditional jump                 |
| jump_if_equal to "name"         | Jump if last compare was equal     |
| jump_if_not_equal to "name"     | Jump if not equal                  |
| jump_if_greater to "name"       | Jump if greater                    |
| jump_if_less to "name"          | Jump if less                       |

### Inline Conditionals
| Syntax                                   | Description                       |
|------------------------------------------|-----------------------------------|
| if tray1 == 42 then tray2 = "Found!"    | Full inline conditional one-liner |
| if tray1 > 100 then tray1 = 100         | Clamp value inline                |
| if tray2 == 'Z' then say "Got Z!"       | Character comparison              |

### Storeroom (RAM)
| Syntax                          | Description                           |
|---------------------------------|---------------------------------------|
| store tray1 into storeroom[0]   | Write tray1 to RAM bin 0             |
| load storeroom[0] into tray2    | Read RAM bin 0 into tray2            |

### Output and Supervisor
| Syntax                 | Description                              |
|------------------------|------------------------------------------|
| say "Hello!"           | Print a string literal to stdout         |
| say tray4              | Print the string stored in tray4         |
| call_supervisor exit   | Clean program exit                       |

### Workstations
| Syntax                      | Description                            |
|-----------------------------|----------------------------------------|
| workstation "name":         | Begin a named workstation              |
| end                         | End current workstation                |
| call workstation "name"     | Call a workstation                     |
| return                      | Return from workstation                |
| return tray1                | Return from workstation with value     |

---

## How FAL JIT Compiles Your Code

1. Parse: The FAL parser reads your .fal file line by line and builds an instruction list.
2. SSL Map: Virtual trays (tray1 to tray14) are mapped to physical registers by the Smart Shuffler Layer based on the live-detected CPU and OS.
3. JIT Emit: The JIT engine writes raw x86_64 machine code bytes directly into an anonymous executable memory page.
4. Patch: String addresses, workstation jump targets, and call offsets are resolved and patched.
5. Execute: The native machine code function is called directly. Zero VM. Zero interpreter. Pure silicon.

---

## Is FAL Turing Complete?

Yes. FAL is mathematically Turing complete because it provides:
- Arbitrary mutable state (14 virtual trays + unlimited storeroom bins)
- Full arithmetic and logic operations
- Unconditional and conditional branching (loops via jumps)
- Subroutine call and return stack
- OS-level I/O via supervisor calls

Any computable algorithm can be written in FAL.

---

## Roadmap

- [x] x86_64 Linux JIT engine
- [x] Smart Shuffler Layer (SSL) for virtual tray mapping
- [x] String and character literals
- [x] Inline conditionals
- [x] Workstations with isolated scopes and private trays
- [x] Storeroom RAM load and store
- [x] FALZ global environment and storage system
- [x] Universal install.sh for Linux and macOS
- [ ] ARM64 / Apple Silicon native JIT emitter
- [ ] Floating-point trays (float_tray1 = 3.14)
- [ ] File I/O supervisor calls
- [ ] Network supervisor calls
- [ ] FAL Web Playground with visual workbench and step debugger
- [ ] The FAL Book: "The Factory Inside Your Computer"
- [ ] Windows installer (.exe and PowerShell script)

---

## Contributing

Contributions of all kinds are welcome: bug reports, feature ideas, documentation, new examples, and code.

Please read [CONTRIBUTING.md](CONTRIBUTING.md) and [CODE_OF_CONDUCT.md](CODE_OF_CONDUCT.md) before opening a pull request.

---

## License

FAL is free and open-source software licensed under the [MIT License](LICENSE).

Created with the goal of making low-level computing education accessible to everyone in the world,
regardless of background, language, or experience level.

---

## Author

Kasish
Creator of Factory Assembly Language (FAL)

> "If a 10th grader can understand how a CPU works through this language, then we have succeeded."
