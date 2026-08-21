#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CpuArch {
    X86_64,
    AArch64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OsSupervisor {
    Linux,
    MacOS,
    Windows,
}

#[derive(Debug, Clone)]
pub struct TargetContext {
    pub arch: CpuArch,
    pub os: OsSupervisor,
    pub available_registers: Vec<&'static str>,
    pub max_physical_trays: usize,
}

impl TargetContext {
    pub fn autodetect() -> Self {
        let arch = if cfg!(target_arch = "x86_64") {
            CpuArch::X86_64
        } else if cfg!(target_arch = "aarch64") {
            CpuArch::AArch64
        } else {
            panic!("Unsupported host CPU architecture! FAL supports x86_64 and AArch64.");
        };

        let os = if cfg!(target_os = "linux") {
            OsSupervisor::Linux
        } else if cfg!(target_os = "macos") {
            OsSupervisor::MacOS
        } else if cfg!(target_os = "windows") {
            OsSupervisor::Windows
        } else {
            panic!("Unsupported host OS supervisor! FAL supports Linux, macOS, and Windows.");
        };

        let available_registers = match arch {
            CpuArch::X86_64 => vec![
                "rax", "rbx", "rcx", "rdx", "rsi", "rdi", "r8", "r9", "r10", "r11", "r12",
                "r13", "r14", "r15",
            ],
            CpuArch::AArch64 => vec![
                "x0", "x1", "x2", "x3", "x4", "x5", "x6", "x7", "x8", "x9", "x10", "x11", "x12",
                "x13", "x14", "x15",
            ],
        };

        let max_physical_trays = available_registers.len();

        Self {
            arch,
            os,
            available_registers,
            max_physical_trays,
        }
    }
}

