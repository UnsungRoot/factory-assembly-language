use crate::target::TargetContext;

#[allow(dead_code)]
pub struct TrayMapper {
    ctx: TargetContext,
}

impl TrayMapper {
    pub fn new(ctx: TargetContext) -> Self {
        Self { ctx }
    }

    #[allow(dead_code)]
    pub fn resolve_register_name(&self, tray_index: usize) -> &'static str {
        let count = self.ctx.available_registers.len();
        self.ctx.available_registers[tray_index % count]
    }


    /// Smart Shuffler Layer (SSL): Maps virtual trays to x86_64 physical hardware registers.
    /// Returns the raw hardware register index (0..15).
    pub fn resolve_x86_reg_id(&self, tray_index: usize) -> u8 {
        match tray_index % 14 {
            0 => 0,  // RAX (tray1) - Primary Return & Accumulator
            1 => 3,  // RBX (tray2) - General Purpose
            2 => 1,  // RCX (tray3) - Counter
            3 => 2,  // RDX (tray4) - Data / Strings
            4 => 6,  // RSI (tray5) - Source Buffer
            5 => 7,  // RDI (tray6) - Destination
            6 => 8,  // R8  (tray7) - Extended Tray 1
            7 => 9,  // R9  (tray8) - Extended Tray 2
            8 => 10, // R10 (tray9) - Extended Tray 3
            9 => 11, // R11 (tray10) - Extended Tray 4
            10 => 12,// R12 (tray11) - Callee Saved 1
            11 => 13,// R13 (tray12) - Callee Saved 2
            12 => 14,// R14 (tray13) - Callee Saved 3
            13 => 15,// R15 (tray14) - Callee Saved 4
            _ => 0,
        }
    }
}

