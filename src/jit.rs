use crate::mapper::TrayMapper;
use crate::parser::{ConditionOp, Instruction, Operand};
use crate::target::{CpuArch, TargetContext};
use memmap2::MmapMut;
use std::collections::HashMap;

struct PendingPatch {
    code_offset: usize,
    target: String,
}

struct PendingStringLoad {
    imm_offset: usize,
    data_offset: usize,
}

pub struct DynamicJitEngine {
    ctx: TargetContext,
    mapper: TrayMapper,
    code: Vec<u8>,
    data_section: Vec<u8>,
    workstation_offsets: HashMap<String, usize>,
    pending_calls: Vec<PendingPatch>,
    pending_jumps: Vec<PendingPatch>,
    pending_string_loads: Vec<PendingStringLoad>,
    entry_jump_offset: Option<usize>,
    current_workstation: Option<String>,
}

impl DynamicJitEngine {
    pub fn new() -> Self {
        let ctx = TargetContext::autodetect();
        let mapper = TrayMapper::new(ctx.clone());

        let mut engine = Self {
            ctx,
            mapper,
            code: Vec::new(),
            data_section: Vec::new(),
            workstation_offsets: HashMap::new(),
            pending_calls: Vec::new(),
            pending_jumps: Vec::new(),
            pending_string_loads: Vec::new(),
            entry_jump_offset: None,
            current_workstation: None,
        };

        engine.emit_prologue();
        engine
    }

    fn emit_prologue(&mut self) {
        if self.ctx.arch == CpuArch::X86_64 {
            // Entry trampoline: reserve space for jump to "main" if needed
            self.code.extend_from_slice(&[
                0x55, // push rbp
                0x48, 0x89, 0xe5, // mov rbp, rsp
                0x48, 0x81, 0xec, 0x00, 0x10, 0x00, 0x00, // sub rsp, 4096 (Storeroom allocation)
            ]);
            // Reserve 5 bytes for jmp to main workstation
            self.code.push(0xe9); // JMP rel32
            self.entry_jump_offset = Some(self.code.len());
            self.code.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
        }
    }

    // === X86_64 INSTRUCTION ENCODING HELPERS ===

    fn emit_mov_reg_imm64(&mut self, reg_id: u8, val: u64) {
        let rex = 0x48 | if reg_id >= 8 { 1 } else { 0 };
        self.code.extend_from_slice(&[rex, 0xb8 + (reg_id % 8)]);
        self.code.extend_from_slice(&val.to_le_bytes());
    }

    fn emit_mov_reg_reg(&mut self, dst_reg: u8, src_reg: u8) {
        let rex = 0x48 | (if src_reg >= 8 { 4 } else { 0 }) | (if dst_reg >= 8 { 1 } else { 0 });
        let modrm = 0xc0 | ((src_reg % 8) << 3) | (dst_reg % 8);
        self.code.extend_from_slice(&[rex, 0x89, modrm]);
    }

    fn emit_add_reg_reg(&mut self, dst_reg: u8, src_reg: u8) {
        let rex = 0x48 | (if src_reg >= 8 { 4 } else { 0 }) | (if dst_reg >= 8 { 1 } else { 0 });
        let modrm = 0xc0 | ((src_reg % 8) << 3) | (dst_reg % 8);
        self.code.extend_from_slice(&[rex, 0x01, modrm]);
    }

    fn emit_add_reg_imm(&mut self, dst_reg: u8, val: u64) {
        if val <= i32::MAX as u64 {
            let rex = 0x48 | if dst_reg >= 8 { 1 } else { 0 };
            let modrm = 0xc0 | (0 << 3) | (dst_reg % 8);
            self.code.extend_from_slice(&[rex, 0x81, modrm]);
            self.code.extend_from_slice(&(val as u32).to_le_bytes());
        } else {
            let temp_reg = 11; // R11
            self.emit_mov_reg_imm64(temp_reg, val);
            self.emit_add_reg_reg(dst_reg, temp_reg);
        }
    }

    fn emit_sub_reg_reg(&mut self, dst_reg: u8, src_reg: u8) {
        let rex = 0x48 | (if src_reg >= 8 { 4 } else { 0 }) | (if dst_reg >= 8 { 1 } else { 0 });
        let modrm = 0xc0 | ((src_reg % 8) << 3) | (dst_reg % 8);
        self.code.extend_from_slice(&[rex, 0x29, modrm]);
    }

    fn emit_sub_reg_imm(&mut self, dst_reg: u8, val: u64) {
        if val <= i32::MAX as u64 {
            let rex = 0x48 | if dst_reg >= 8 { 1 } else { 0 };
            let modrm = 0xc0 | (5 << 3) | (dst_reg % 8);
            self.code.extend_from_slice(&[rex, 0x81, modrm]);
            self.code.extend_from_slice(&(val as u32).to_le_bytes());
        } else {
            let temp_reg = 11; // R11
            self.emit_mov_reg_imm64(temp_reg, val);
            self.emit_sub_reg_reg(dst_reg, temp_reg);
        }
    }

    fn emit_imul_reg_reg(&mut self, dst_reg: u8, src_reg: u8) {
        let rex = 0x48 | (if dst_reg >= 8 { 4 } else { 0 }) | (if src_reg >= 8 { 1 } else { 0 });
        let modrm = 0xc0 | ((dst_reg % 8) << 3) | (src_reg % 8);
        self.code.extend_from_slice(&[rex, 0x0f, 0xaf, modrm]);
    }

    fn emit_imul_reg_imm(&mut self, dst_reg: u8, val: u64) {
        let temp_reg = 11; // R11
        self.emit_mov_reg_imm64(temp_reg, val);
        self.emit_imul_reg_reg(dst_reg, temp_reg);
    }

    fn emit_divide_reg_reg(&mut self, dst_reg: u8, src_reg: u8) {
        // rax = dst_reg; rdx = 0; idiv src_reg; dst_reg = rax
        self.emit_mov_reg_reg(0, dst_reg); // mov rax, dst_reg
        self.emit_mov_reg_imm64(2, 0);      // mov rdx, 0
        let rex = 0x48 | if src_reg >= 8 { 1 } else { 0 };
        let modrm = 0xc0 | (7 << 3) | (src_reg % 8);
        self.code.extend_from_slice(&[rex, 0xf7, modrm]); // idiv src_reg
        self.emit_mov_reg_reg(dst_reg, 0); // mov dst_reg, rax
    }

    fn emit_cmp_reg_reg(&mut self, reg_a: u8, reg_b: u8) {
        let rex = 0x48 | (if reg_b >= 8 { 4 } else { 0 }) | (if reg_a >= 8 { 1 } else { 0 });
        let modrm = 0xc0 | ((reg_b % 8) << 3) | (reg_a % 8);
        self.code.extend_from_slice(&[rex, 0x39, modrm]);
    }

    fn emit_cmp_reg_imm(&mut self, reg_a: u8, val: u64) {
        if val <= i32::MAX as u64 {
            let rex = 0x48 | if reg_a >= 8 { 1 } else { 0 };
            let modrm = 0xc0 | (7 << 3) | (reg_a % 8);
            self.code.extend_from_slice(&[rex, 0x81, modrm]);
            self.code.extend_from_slice(&(val as u32).to_le_bytes());
        } else {
            let temp_reg = 11;
            self.emit_mov_reg_imm64(temp_reg, val);
            self.emit_cmp_reg_reg(reg_a, temp_reg);
        }
    }

    // === COMPILE INSTRUCTIONS ===

    pub fn compile_instruction(&mut self, inst: &Instruction) {
        match inst {
            Instruction::Workstation { name } => {
                if let Some(prev) = &self.current_workstation {
                    if prev == "main" {
                        self.code.push(0xe9); // JMP rel32 to __fal_exit
                        let code_offset = self.code.len();
                        self.code.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
                        self.pending_jumps.push(PendingPatch {
                            code_offset,
                            target: "__fal_exit".to_string(),
                        });
                    } else {
                        self.code.push(0xc3); // ret
                    }
                }
                self.current_workstation = Some(name.clone());
                self.workstation_offsets
                    .insert(name.clone(), self.code.len());
            }
            Instruction::EndWorkstation => {
                if self.current_workstation.as_deref() == Some("main") {
                    self.code.push(0xe9); // JMP rel32 to __fal_exit
                    let code_offset = self.code.len();
                    self.code.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
                    self.pending_jumps.push(PendingPatch {
                        code_offset,
                        target: "__fal_exit".to_string(),
                    });
                } else {
                    self.code.push(0xc3); // ret
                }
                self.current_workstation = None;
            }

            Instruction::Fill { val, tray } => {
                let reg_id = self.mapper.resolve_x86_reg_id(*tray);
                self.emit_mov_reg_imm64(reg_id, *val);
            }
            Instruction::AssignChar { val, tray } => {
                let reg_id = self.mapper.resolve_x86_reg_id(*tray);
                self.emit_mov_reg_imm64(reg_id, *val as u64);
            }
            Instruction::AssignString { text, tray } => {
                let reg_id = self.mapper.resolve_x86_reg_id(*tray);
                let data_offset = self.data_section.len();
                self.data_section.extend_from_slice(text.as_bytes());
                self.data_section.push(0); // null terminator

                // Emit mov reg, imm64 (placeholder address)
                let rex = 0x48 | if reg_id >= 8 { 1 } else { 0 };
                self.code.extend_from_slice(&[rex, 0xb8 + (reg_id % 8)]);
                let imm_offset = self.code.len();
                self.code.extend_from_slice(&[0x00; 8]);

                self.pending_string_loads.push(PendingStringLoad {
                    imm_offset,
                    data_offset,
                });
            }
            Instruction::Move { src_tray, dst_tray } => {
                let src_reg = self.mapper.resolve_x86_reg_id(*src_tray);
                let dst_reg = self.mapper.resolve_x86_reg_id(*dst_tray);
                self.emit_mov_reg_reg(dst_reg, src_reg);
            }
            Instruction::Clear { tray } => {
                let reg_id = self.mapper.resolve_x86_reg_id(*tray);
                self.emit_mov_reg_imm64(reg_id, 0);
            }
            Instruction::Add { src, dst_tray } => {
                let dst_reg = self.mapper.resolve_x86_reg_id(*dst_tray);
                match src {
                    Operand::Tray(t) => {
                        let src_reg = self.mapper.resolve_x86_reg_id(*t);
                        self.emit_add_reg_reg(dst_reg, src_reg);
                    }
                    Operand::Number(n) => self.emit_add_reg_imm(dst_reg, *n),
                    Operand::Char(c) => self.emit_add_reg_imm(dst_reg, *c as u64),
                    _ => {}
                }
            }
            Instruction::Sub { src, dst_tray } => {
                let dst_reg = self.mapper.resolve_x86_reg_id(*dst_tray);
                match src {
                    Operand::Tray(t) => {
                        let src_reg = self.mapper.resolve_x86_reg_id(*t);
                        self.emit_sub_reg_reg(dst_reg, src_reg);
                    }
                    Operand::Number(n) => self.emit_sub_reg_imm(dst_reg, *n),
                    Operand::Char(c) => self.emit_sub_reg_imm(dst_reg, *c as u64),
                    _ => {}
                }
            }
            Instruction::Multiply { src, dst_tray } => {
                let dst_reg = self.mapper.resolve_x86_reg_id(*dst_tray);
                match src {
                    Operand::Tray(t) => {
                        let src_reg = self.mapper.resolve_x86_reg_id(*t);
                        self.emit_imul_reg_reg(dst_reg, src_reg);
                    }
                    Operand::Number(n) => self.emit_imul_reg_imm(dst_reg, *n),
                    Operand::Char(c) => self.emit_imul_reg_imm(dst_reg, *c as u64),
                    _ => {}
                }
            }
            Instruction::Divide { src, dst_tray } => {
                let dst_reg = self.mapper.resolve_x86_reg_id(*dst_tray);
                match src {
                    Operand::Tray(t) => {
                        let src_reg = self.mapper.resolve_x86_reg_id(*t);
                        self.emit_divide_reg_reg(dst_reg, src_reg);
                    }
                    Operand::Number(n) => {
                        let temp_reg = 11;
                        self.emit_mov_reg_imm64(temp_reg, *n);
                        self.emit_divide_reg_reg(dst_reg, temp_reg);
                    }
                    _ => {}
                }
            }
            Instruction::Compare { tray_a, val_b } => {
                let reg_a = self.mapper.resolve_x86_reg_id(*tray_a);
                match val_b {
                    Operand::Tray(t) => {
                        let reg_b = self.mapper.resolve_x86_reg_id(*t);
                        self.emit_cmp_reg_reg(reg_a, reg_b);
                    }
                    Operand::Number(n) => self.emit_cmp_reg_imm(reg_a, *n),
                    Operand::Char(c) => self.emit_cmp_reg_imm(reg_a, *c as u64),
                    _ => {}
                }
            }
            Instruction::Jump { target } => {
                self.code.push(0xe9); // JMP rel32
                let code_offset = self.code.len();
                self.code.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
                self.pending_jumps.push(PendingPatch {
                    code_offset,
                    target: target.clone(),
                });
            }
            Instruction::JumpIfEqual { target } => {
                self.code.extend_from_slice(&[0x0f, 0x84]); // JE rel32
                let code_offset = self.code.len();
                self.code.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
                self.pending_jumps.push(PendingPatch {
                    code_offset,
                    target: target.clone(),
                });
            }
            Instruction::JumpIfNotEqual { target } => {
                self.code.extend_from_slice(&[0x0f, 0x85]); // JNE rel32
                let code_offset = self.code.len();
                self.code.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
                self.pending_jumps.push(PendingPatch {
                    code_offset,
                    target: target.clone(),
                });
            }
            Instruction::JumpIfGreater { target } => {
                self.code.extend_from_slice(&[0x0f, 0x8f]); // JG rel32
                let code_offset = self.code.len();
                self.code.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
                self.pending_jumps.push(PendingPatch {
                    code_offset,
                    target: target.clone(),
                });
            }
            Instruction::JumpIfLess { target } => {
                self.code.extend_from_slice(&[0x0f, 0x8c]); // JL rel32
                let code_offset = self.code.len();
                self.code.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
                self.pending_jumps.push(PendingPatch {
                    code_offset,
                    target: target.clone(),
                });
            }
            Instruction::InlineIf {
                tray,
                op,
                target_val,
                then_inst,
            } => {
                // 1. Emit comparison
                let reg_a = self.mapper.resolve_x86_reg_id(*tray);
                match target_val {
                    Operand::Tray(t) => {
                        let reg_b = self.mapper.resolve_x86_reg_id(*t);
                        self.emit_cmp_reg_reg(reg_a, reg_b);
                    }
                    Operand::Number(n) => self.emit_cmp_reg_imm(reg_a, *n),
                    Operand::Char(c) => self.emit_cmp_reg_imm(reg_a, *c as u64),
                    _ => {}
                }

                // 2. Emit inverted conditional jump skipping then_inst
                // e.g. If op == Equal, jump over when NOT equal (JNE)
                match op {
                    ConditionOp::Equal => self.code.extend_from_slice(&[0x0f, 0x85]), // JNE
                    ConditionOp::NotEqual => self.code.extend_from_slice(&[0x0f, 0x84]), // JE
                    ConditionOp::Greater => self.code.extend_from_slice(&[0x0f, 0x8e]), // JLE
                    ConditionOp::Less => self.code.extend_from_slice(&[0x0f, 0x8d]), // JGE
                    ConditionOp::GreaterEqual => self.code.extend_from_slice(&[0x0f, 0x8c]), // JL
                    ConditionOp::LessEqual => self.code.extend_from_slice(&[0x0f, 0x8f]), // JG
                }
                let skip_patch_offset = self.code.len();
                self.code.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);

                // 3. Compile then_inst
                self.compile_instruction(then_inst);

                // 4. Patch skip jump to point right after then_inst
                let skip_dest = self.code.len();
                let rel = (skip_dest as i64) - ((skip_patch_offset + 4) as i64);
                let bytes = (rel as i32).to_le_bytes();
                self.code[skip_patch_offset..skip_patch_offset + 4].copy_from_slice(&bytes);
            }
            Instruction::Store { tray, offset } => {
                let reg_id = self.mapper.resolve_x86_reg_id(*tray);
                let disp = ((*offset + 1) * 8) as i32;
                let rex = 0x48 | if reg_id >= 8 { 4 } else { 0 };
                let modrm = 0x85 | ((reg_id % 8) << 3); // [rbp + disp32]
                self.code.extend_from_slice(&[rex, 0x89, modrm]);
                self.code.extend_from_slice(&(-disp).to_le_bytes());
            }
            Instruction::Load { offset, tray } => {
                let reg_id = self.mapper.resolve_x86_reg_id(*tray);
                let disp = ((*offset + 1) * 8) as i32;
                let rex = 0x48 | if reg_id >= 8 { 4 } else { 0 };
                let modrm = 0x85 | ((reg_id % 8) << 3); // [rbp + disp32]
                self.code.extend_from_slice(&[rex, 0x8b, modrm]);
                self.code.extend_from_slice(&(-disp).to_le_bytes());
            }
            Instruction::SayLiteral { text } => {
                let data_offset = self.data_section.len();
                let mut full_text = text.clone();
                full_text.push('\n');
                let bytes = full_text.as_bytes();
                let len = bytes.len();
                self.data_section.extend_from_slice(bytes);

                // Preserve registers across supervisor call (SSL register protection)
                self.emit_preserve_registers();

                // Linux sys_write (RAX=1, RDI=1, RSI=str_ptr, RDX=len)
                self.emit_mov_reg_imm64(0, 1); // RAX = 1
                self.emit_mov_reg_imm64(7, 1); // RDI = 1

                // MOV RSI, imm64
                self.code.extend_from_slice(&[0x48, 0xbe]); // MOV RSI, imm64
                let imm_offset = self.code.len();
                self.code.extend_from_slice(&[0x00; 8]);
                self.pending_string_loads.push(PendingStringLoad {
                    imm_offset,
                    data_offset,
                });

                // MOV RDX, len
                self.emit_mov_reg_imm64(2, len as u64); // RDX = len

                // Syscall
                self.code.extend_from_slice(&[0x0f, 0x05]);

                // Restore registers
                self.emit_restore_registers();
            }
            Instruction::SayTray { tray } => {
                let reg_id = self.mapper.resolve_x86_reg_id(*tray);

                self.emit_preserve_registers();

                // Load string pointer from tray into RSI
                self.emit_mov_reg_reg(6, reg_id); // RSI = tray_reg

                // Find strlen dynamically in assembly (scan for null terminator)
                // mov rdi, rsi; xor al, al; mov rcx, 1024; repne scasb; sub rdi, rsi; dec rdi; mov rdx, rdi
                self.code.extend_from_slice(&[
                    0x48, 0x89, 0xf7, // mov rdi, rsi
                    0x30, 0xc0,       // xor al, al
                    0x48, 0xc7, 0xc1, 0x00, 0x04, 0x00, 0x00, // mov rcx, 1024
                    0xf2, 0xae,       // repne scasb
                    0x48, 0x29, 0xf7, // sub rdi, rsi
                    0x48, 0xff, 0xcf, // dec rdi
                    0x48, 0x89, 0xfa, // mov rdx, rdi (length)
                    0x48, 0xc7, 0xc0, 0x01, 0x00, 0x00, 0x00, // mov rax, 1 (sys_write)
                    0x48, 0xc7, 0xc7, 0x01, 0x00, 0x00, 0x00, // mov rdi, 1 (stdout)
                    0x0f, 0x05,       // syscall
                ]);

                // Print trailing newline
                let nl_offset = self.data_section.len();
                self.data_section.push(b'\n');
                self.code.extend_from_slice(&[
                    0x48, 0xc7, 0xc0, 0x01, 0x00, 0x00, 0x00, // mov rax, 1
                    0x48, 0xc7, 0xc7, 0x01, 0x00, 0x00, 0x00, // mov rdi, 1
                    0x48, 0xbe, // mov rsi, imm64
                ]);
                let imm_offset = self.code.len();
                self.code.extend_from_slice(&[0x00; 8]);
                self.pending_string_loads.push(PendingStringLoad {
                    imm_offset,
                    data_offset: nl_offset,
                });
                self.code.extend_from_slice(&[
                    0x48, 0xc7, 0xc2, 0x01, 0x00, 0x00, 0x00, // mov rdx, 1
                    0x0f, 0x05, // syscall
                ]);

                self.emit_restore_registers();
            }
            Instruction::Call { workstation } => {
                self.code.push(0xe8); // CALL rel32
                let code_offset = self.code.len();
                self.code.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
                self.pending_calls.push(PendingPatch {
                    code_offset,
                    target: workstation.clone(),
                });
            }
            Instruction::Return { tray } => {
                if let Some(t) = tray {
                    let ret_reg = self.mapper.resolve_x86_reg_id(*t);
                    if ret_reg != 0 {
                        self.emit_mov_reg_reg(0, ret_reg); // mov rax, ret_reg
                    }
                }
                self.code.push(0xc3); // RET
            }
            Instruction::CallSupervisor { action, tray } => {
                if action == "exit" {
                    self.code.push(0xe9); // JMP rel32 to __fal_exit
                    let code_offset = self.code.len();
                    self.code.extend_from_slice(&[0x00, 0x00, 0x00, 0x00]);
                    self.pending_jumps.push(PendingPatch {
                        code_offset,
                        target: "__fal_exit".to_string(),
                    });
                } else if action == "print" {
                    if let Some(t) = tray {
                        self.compile_instruction(&Instruction::SayTray { tray: *t });
                    }
                }
            }
        }
    }

    fn emit_preserve_registers(&mut self) {
        // Push registers that syscalls or subroutines could clobber
        self.code.extend_from_slice(&[
            0x50, // push rax
            0x51, // push rcx
            0x52, // push rdx
            0x56, // push rsi
            0x57, // push rdi
            0x41, 0x53, // push r11
        ]);
    }

    fn emit_restore_registers(&mut self) {
        self.code.extend_from_slice(&[
            0x41, 0x5b, // pop r11
            0x5f, // pop rdi
            0x5e, // pop rsi
            0x5a, // pop rdx
            0x59, // pop rcx
            0x58, // pop rax
        ]);
    }

    fn patch_targets(&mut self, base_ptr: usize) {
        let code_len = self.code.len();
        let data_base = base_ptr + code_len;

        // 1. Patch entry jump to "main" workstation
        if let Some(entry_offset) = self.entry_jump_offset {
            let target_offset = self
                .workstation_offsets
                .get("main")
                .cloned()
                .unwrap_or(entry_offset + 4);
            let rel = (target_offset as i64) - ((entry_offset + 4) as i64);
            let bytes = (rel as i32).to_le_bytes();
            self.code[entry_offset..entry_offset + 4].copy_from_slice(&bytes);
        }

        // 2. Patch calls and jumps
        for patch in self.pending_calls.iter().chain(self.pending_jumps.iter()) {
            if let Some(&target_offset) = self.workstation_offsets.get(&patch.target) {
                let next_inst = patch.code_offset + 4;
                let rel_offset = (target_offset as i64) - (next_inst as i64);
                let bytes = (rel_offset as i32).to_le_bytes();
                self.code[patch.code_offset..patch.code_offset + 4].copy_from_slice(&bytes);
            }
        }

        // 3. Patch string absolute addresses
        for str_load in &self.pending_string_loads {
            let abs_addr = (data_base + str_load.data_offset) as u64;
            self.code[str_load.imm_offset..str_load.imm_offset + 8]
                .copy_from_slice(&abs_addr.to_le_bytes());
        }
    }

    pub fn print_debugger_state(&self) {
        println!("\n=== [FAL VISUAL DEBUGGER] ===");
        println!("JIT Native Code Size : {} bytes", self.code.len());
        println!("Data Section Size    : {} bytes", self.data_section.len());
        println!(
            "Workstations Active  : {:?}",
            self.workstation_offsets.keys().collect::<Vec<_>>()
        );
        println!("=============================\n");
    }

    pub fn execute(mut self, debug: bool) -> u64 {
        // Global exit point for all workstations
        let exit_offset = self.code.len();
        self.workstation_offsets
            .insert("__fal_exit".to_string(), exit_offset);

        // Epilogue
        self.code.extend_from_slice(&[
            0x48, 0x81, 0xc4, 0x00, 0x10, 0x00, 0x00, // add rsp, 4096
            0x5d, // pop rbp
            0xc3, // ret
        ]);

        let total_size = (self.code.len() + self.data_section.len() + 4095) & !4095;
        let mut mmap = MmapMut::map_anon(total_size.max(4096)).expect("Alloc RWX memory failed");

        let base_ptr = mmap.as_ptr() as usize;
        self.patch_targets(base_ptr);

        if debug {
            self.print_debugger_state();
        }

        // Copy code and data section into executable buffer
        let code_len = self.code.len();
        mmap[..code_len].copy_from_slice(&self.code);
        if !self.data_section.is_empty() {
            mmap[code_len..code_len + self.data_section.len()]
                .copy_from_slice(&self.data_section);
        }

        let executable = mmap.make_exec().expect("Make memory exec failed");
        let jit_func: fn() -> u64 = unsafe { std::mem::transmute(executable.as_ptr()) };

        jit_func()
    }
}


