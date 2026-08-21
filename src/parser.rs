#[derive(Debug, PartialEq, Clone)]
pub enum Operand {
    Tray(usize),
    Number(u64),
    Char(char),
    StringLit(String),
}

#[derive(Debug, PartialEq, Clone, Copy)]
pub enum ConditionOp {
    Equal,
    NotEqual,
    Greater,
    Less,
    GreaterEqual,
    LessEqual,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Instruction {
    Workstation { name: String },
    EndWorkstation,
    Fill { val: u64, tray: usize },
    AssignChar { val: char, tray: usize },
    AssignString { text: String, tray: usize },
    Move { src_tray: usize, dst_tray: usize },
    Clear { tray: usize },
    Add { src: Operand, dst_tray: usize },
    Sub { src: Operand, dst_tray: usize },
    Multiply { src: Operand, dst_tray: usize },
    Divide { src: Operand, dst_tray: usize },
    Compare { tray_a: usize, val_b: Operand },
    Jump { target: String },
    JumpIfEqual { target: String },
    JumpIfNotEqual { target: String },
    JumpIfGreater { target: String },
    JumpIfLess { target: String },
    InlineIf {
        tray: usize,
        op: ConditionOp,
        target_val: Operand,
        then_inst: Box<Instruction>,
    },
    Store { tray: usize, offset: usize },
    Load { offset: usize, tray: usize },
    SayLiteral { text: String },
    SayTray { tray: usize },
    Call { workstation: String },
    Return { tray: Option<usize> },
    CallSupervisor { action: String, tray: Option<usize> },
}

pub struct Parser;

impl Parser {
    pub fn parse_line(line: &str) -> Option<Instruction> {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with("//") || trimmed.starts_with('#') {
            return None;
        }

        // Strip inline comments: tray1 = 10 // comment
        let code_line = if let Some(idx) = trimmed.find("//") {
            trimmed[..idx].trim()
        } else if let Some(idx) = trimmed.find('#') {
            trimmed[..idx].trim()
        } else {
            trimmed
        };

        if code_line.is_empty() {
            return None;
        }

        // === INLINE CONDITIONAL: if tray2 == '8' then tray4 = "Hello" ===
        if code_line.to_lowercase().starts_with("if ") {
            if let Some(then_idx) = code_line.to_lowercase().find(" then ") {
                let condition_part = code_line[3..then_idx].trim();
                let then_part = code_line[then_idx + 6..].trim();

                if let Some((tray, op, target_val)) = Self::parse_condition(condition_part) {
                    if let Some(then_inst) = Self::parse_line(then_part) {
                        return Some(Instruction::InlineIf {
                            tray,
                            op,
                            target_val,
                            then_inst: Box::new(then_inst),
                        });
                    }
                }
            }
        }

        // === DIRECT ASSIGNMENT: tray1 = 10, tray2 = '8', tray4 = "Hello, World!" ===
        if code_line.contains('=') && !code_line.contains("==") && !code_line.contains("!=") && !code_line.contains("<=") && !code_line.contains(">=") {
            let parts: Vec<&str> = code_line.splitn(2, '=').map(|s| s.trim()).collect();
            if parts.len() == 2 && parts[0].to_lowercase().starts_with("tray") {
                let tray = parse_tray_index(parts[0]);
                let val_str = parts[1];

                // String literal: tray4 = "Hello, World!"
                if val_str.starts_with('"') && val_str.ends_with('"') && val_str.len() >= 2 {
                    let text = val_str[1..val_str.len() - 1].to_string();
                    return Some(Instruction::AssignString { text, tray });
                }

                // Character literal: tray2 = '8'
                if val_str.starts_with('\'') && val_str.ends_with('\'') && val_str.len() >= 3 {
                    let ch = val_str.chars().nth(1).unwrap_or(' ');
                    return Some(Instruction::AssignChar { val: ch, tray });
                }

                // Numeric literal: tray1 = 42
                if let Ok(val) = val_str.parse::<u64>() {
                    return Some(Instruction::Fill { val, tray });
                }
            }
        }

        // === SAY LITERAL: say "Hello, World!" ===
        if code_line.to_lowercase().starts_with("say ") {
            let rest = code_line[4..].trim();
            if rest.starts_with('"') && rest.ends_with('"') && rest.len() >= 2 {
                let text = rest[1..rest.len() - 1].to_string();
                return Some(Instruction::SayLiteral { text });
            } else if rest.to_lowercase().starts_with("tray") {
                let tray = parse_tray_index(rest);
                return Some(Instruction::SayTray { tray });
            }
        }

        let tokens: Vec<&str> = code_line.split_whitespace().collect();
        if tokens.is_empty() {
            return None;
        }

        let first = tokens[0].to_lowercase();
        match first.as_str() {
            "workstation" if tokens.len() >= 2 => {
                let raw_name = tokens[1..].join(" ");
                let name = sanitize_name(&raw_name);
                Some(Instruction::Workstation { name })
            }
            "end" | "endworkstation" | "exit_workstation" => Some(Instruction::EndWorkstation),
            "move" | "copy" if tokens.len() >= 4 && tokens[2].to_lowercase() == "to" => {
                // move tray1 to tray2  OR  move 10 to tray1 OR move "text" to tray4
                let src_str = tokens[1];
                let dst_tray = parse_tray_index(tokens[3]);
                if src_str.starts_with('"') {
                    let rest = tokens[1..].join(" ");
                    if let Some(first_q) = rest.find('"') {
                        if let Some(last_q) = rest.rfind('"') {
                            if last_q > first_q {
                                let text = rest[first_q + 1..last_q].to_string();
                                return Some(Instruction::AssignString { text, tray: dst_tray });
                            }
                        }
                    }
                }
                if src_str.to_lowercase().starts_with("tray") {
                    let src_tray = parse_tray_index(src_str);
                    Some(Instruction::Move { src_tray, dst_tray })
                } else if let Ok(val) = src_str.parse::<u64>() {
                    Some(Instruction::Fill { val, tray: dst_tray })
                } else {
                    None
                }
            }
            "fill" if tokens.len() >= 4 && tokens[2].to_lowercase() == "into" => {
                let val = tokens[1].parse::<u64>().unwrap_or(0);
                let tray = parse_tray_index(tokens[3]);
                Some(Instruction::Fill { val, tray })
            }
            "clear" if tokens.len() >= 2 => {
                let tray = parse_tray_index(tokens[1]);
                Some(Instruction::Clear { tray })
            }
            "add" if tokens.len() >= 4 && tokens[2].to_lowercase() == "to" => {
                let src = parse_operand(tokens[1]);
                let dst_tray = parse_tray_index(tokens[3]);
                Some(Instruction::Add { src, dst_tray })
            }
            "sub" if tokens.len() >= 4 && tokens[2].to_lowercase() == "from" => {
                let src = parse_operand(tokens[1]);
                let dst_tray = parse_tray_index(tokens[3]);
                Some(Instruction::Sub { src, dst_tray })
            }
            "multiply" if tokens.len() >= 4 && tokens[2].to_lowercase() == "by" => {
                let dst_tray = parse_tray_index(tokens[1]);
                let src = parse_operand(tokens[3]);
                Some(Instruction::Multiply { src, dst_tray })
            }
            "divide" if tokens.len() >= 4 && tokens[2].to_lowercase() == "by" => {
                let dst_tray = parse_tray_index(tokens[1]);
                let src = parse_operand(tokens[3]);
                Some(Instruction::Divide { src, dst_tray })
            }
            "compare" if tokens.len() >= 4 && tokens[2].to_lowercase() == "with" => {
                let tray_a = parse_tray_index(tokens[1]);
                let val_b = parse_operand(tokens[3]);
                Some(Instruction::Compare { tray_a, val_b })
            }
            "jump" if tokens.len() >= 2 => {
                let target = sanitize_name(tokens.last().unwrap());
                Some(Instruction::Jump { target })
            }
            "jump_if_equal" | "je" => {
                let target = sanitize_name(tokens.last().unwrap());
                Some(Instruction::JumpIfEqual { target })
            }
            "jump_if_not_equal" | "jne" => {
                let target = sanitize_name(tokens.last().unwrap());
                Some(Instruction::JumpIfNotEqual { target })
            }
            "jump_if_greater" | "jg" => {
                let target = sanitize_name(tokens.last().unwrap());
                Some(Instruction::JumpIfGreater { target })
            }
            "jump_if_less" | "jl" => {
                let target = sanitize_name(tokens.last().unwrap());
                Some(Instruction::JumpIfLess { target })
            }
            "store" if tokens.len() >= 3 => {
                let tray = parse_tray_index(tokens[1]);
                let offset = parse_storeroom_offset(&tokens[2..].join(" "));
                Some(Instruction::Store { tray, offset })
            }
            "load" | "fetch" if tokens.len() >= 3 => {
                let offset = parse_storeroom_offset(&tokens[1..tokens.len() - 1].join(" "));
                let tray = parse_tray_index(tokens.last().unwrap());
                Some(Instruction::Load { offset, tray })
            }
            "call" if tokens.len() >= 2 => {
                if tokens[1].to_lowercase() == "supervisor" || tokens[1].to_lowercase() == "call_supervisor" {
                    let action = if tokens.len() >= 3 { tokens[2].to_lowercase() } else { "default".to_string() };
                    let tray = if tokens.len() >= 4 { Some(parse_tray_index(tokens[3])) } else { None };
                    Some(Instruction::CallSupervisor { action, tray })
                } else {
                    let target = sanitize_name(tokens.last().unwrap());
                    Some(Instruction::Call { workstation: target })
                }
            }
            "call_supervisor" => {
                let action = if tokens.len() >= 2 { tokens[1].to_lowercase() } else { "default".to_string() };
                let tray = if tokens.len() >= 3 { Some(parse_tray_index(tokens[2])) } else { None };
                Some(Instruction::CallSupervisor { action, tray })
            }
            "return" | "ret" => {
                let tray = if tokens.len() >= 2 {
                    Some(parse_tray_index(tokens[1]))
                } else {
                    None
                };
                Some(Instruction::Return { tray })
            }
            _ => None,
        }
    }

    fn parse_condition(cond: &str) -> Option<(usize, ConditionOp, Operand)> {
        let ops = [
            ("==", ConditionOp::Equal),
            ("!=", ConditionOp::NotEqual),
            (">=", ConditionOp::GreaterEqual),
            ("<=", ConditionOp::LessEqual),
            (">", ConditionOp::Greater),
            ("<", ConditionOp::Less),
        ];

        for (symbol, op) in ops {
            if let Some(idx) = cond.find(symbol) {
                let left_str = cond[..idx].trim();
                let right_str = cond[idx + symbol.len()..].trim();

                if left_str.to_lowercase().starts_with("tray") {
                    let tray = parse_tray_index(left_str);
                    let target_val = parse_operand(right_str);
                    return Some((tray, op, target_val));
                }
            }
        }
        None
    }
}

pub fn parse_tray_index(token: &str) -> usize {
    token
        .trim_matches(|c: char| !c.is_numeric())
        .parse::<usize>()
        .unwrap_or(1)
        .saturating_sub(1)
}

pub fn sanitize_name(s: &str) -> String {
    s.trim()
        .trim_matches(|c: char| c == ':' || c == '"' || c == '\'' || c == ',' || c == ';')
        .trim()
        .to_string()
}

pub fn parse_operand(token: &str) -> Operand {
    let t = token.trim();
    if t.to_lowercase().starts_with("tray") {
        Operand::Tray(parse_tray_index(t))
    } else if t.starts_with('\'') && t.ends_with('\'') && t.len() >= 3 {
        Operand::Char(t.chars().nth(1).unwrap_or(' '))
    } else if t.starts_with('"') && t.ends_with('"') && t.len() >= 2 {
        Operand::StringLit(t[1..t.len() - 1].to_string())
    } else {
        Operand::Number(t.parse::<u64>().unwrap_or(0))
    }
}

pub fn parse_storeroom_offset(s: &str) -> usize {
    if let Some(open) = s.find('[') {
        if let Some(close) = s.find(']') {
            if close > open {
                return s[open + 1..close].trim().parse::<usize>().unwrap_or(0);
            }
        }
    }
    s.trim_matches(|c: char| !c.is_numeric())
        .parse::<usize>()
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_string_assignment() {
        let inst = Parser::parse_line("tray4 = \"Hello, World!\"").unwrap();
        assert_eq!(
            inst,
            Instruction::AssignString {
                text: "Hello, World!".to_string(),
                tray: 3,
            }
        );
    }

    #[test]
    fn test_parse_char_assignment() {
        let inst = Parser::parse_line("tray2 = '8'").unwrap();
        assert_eq!(
            inst,
            Instruction::AssignChar {
                val: '8',
                tray: 1,
            }
        );
    }

    #[test]
    fn test_parse_inline_conditional() {
        let inst = Parser::parse_line("if tray2 == '8' then tray4 = \"Hello, World!\"").unwrap();
        assert_eq!(
            inst,
            Instruction::InlineIf {
                tray: 1,
                op: ConditionOp::Equal,
                target_val: Operand::Char('8'),
                then_inst: Box::new(Instruction::AssignString {
                    text: "Hello, World!".to_string(),
                    tray: 3,
                }),
            }
        );
    }

    #[test]
    fn test_parse_storeroom_and_math() {
        let inst1 = Parser::parse_line("store tray1 into storeroom[0]").unwrap();
        assert_eq!(inst1, Instruction::Store { tray: 0, offset: 0 });

        let inst2 = Parser::parse_line("load storeroom[5] into tray3").unwrap();
        assert_eq!(inst2, Instruction::Load { offset: 5, tray: 2 });

        let inst3 = Parser::parse_line("add 50 to tray1").unwrap();
        assert_eq!(inst3, Instruction::Add { src: Operand::Number(50), dst_tray: 0 });

        let inst4 = Parser::parse_line("say tray4").unwrap();
        assert_eq!(inst4, Instruction::SayTray { tray: 3 });
    }
}


