use crate::target::TargetContext;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct FalzEnvironment {
    pub root_dir: PathBuf,
    pub cache_dir: PathBuf,
    pub storeroom_dir: PathBuf,
    pub logs_dir: PathBuf,
    pub env_file: PathBuf,
}

#[derive(Debug)]
pub struct FalzStats {
    pub root_path: String,
    pub cached_files_count: usize,
    pub log_files_count: usize,
    pub storeroom_slots_count: usize,
    pub total_storage_bytes: u64,
}


impl FalzEnvironment {
    /// Discovers or initializes the global FALZ environment path (~/.falz or $FALZ_HOME)
    pub fn init(ctx: &TargetContext) -> Self {
        let root_dir = if let Ok(custom_path) = std::env::var("FALZ_HOME") {
            PathBuf::from(custom_path)
        } else if let Ok(home) = std::env::var("HOME") {
            PathBuf::from(home).join(".falz")
        } else {
            PathBuf::from(".falz")
        };

        let cache_dir = root_dir.join("cache");
        let storeroom_dir = root_dir.join("storeroom");
        let logs_dir = root_dir.join("logs");
        let env_file = root_dir.join("env.json");

        // Ensure directories exist
        let _ = fs::create_dir_all(&cache_dir);
        let _ = fs::create_dir_all(&storeroom_dir);
        let _ = fs::create_dir_all(&logs_dir);

        let env = Self {
            root_dir,
            cache_dir,
            storeroom_dir,
            logs_dir,
            env_file,
        };

        // Cache hardware detection environment
        env.persist_hardware_context(ctx);

        env
    }

    /// Saves the detected CPU architecture, OS supervisor, and SSL registers into FALZ env.json
    fn persist_hardware_context(&self, ctx: &TargetContext) {
        let regs_json = ctx
            .available_registers
            .iter()
            .map(|r| format!("\"{}\"", r))
            .collect::<Vec<_>>()
            .join(", ");

        let json_content = format!(
            "{{\n  \"engine\": \"Factory Assembly Language (FAL)\",\n  \"environment\": \"FALZ Global Storage\",\n  \"target_arch\": \"{:?}\",\n  \"host_os\": \"{:?}\",\n  \"physical_trays_detected\": {},\n  \"ssl_register_pool\": [{}]\n}}\n",
            ctx.arch, ctx.os, ctx.max_physical_trays, regs_json
        );

        let _ = fs::write(&self.env_file, json_content);
    }

    /// Logs an execution trace or JIT session into FALZ logs
    pub fn save_log(&self, program_name: &str, content: &str) {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let clean_name = Path::new(program_name)
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("fal_run");
        let log_path = self.logs_dir.join(format!("{}_{}.log", clean_name, timestamp));
        let _ = fs::write(log_path, content);
    }

    /// Collects storage statistics for FALZ
    pub fn get_stats(&self) -> FalzStats {
        let count_dir = |dir: &Path| -> (usize, u64) {
            let mut count = 0;
            let mut bytes = 0;
            if let Ok(entries) = fs::read_dir(dir) {
                for entry in entries.flatten() {
                    if let Ok(meta) = entry.metadata() {
                        if meta.is_file() {
                            count += 1;
                            bytes += meta.len();
                        }
                    }
                }
            }
            (count, bytes)
        };

        let (cached_files_count, cache_bytes) = count_dir(&self.cache_dir);
        let (log_files_count, log_bytes) = count_dir(&self.logs_dir);
        let (storeroom_slots_count, storeroom_bytes) = count_dir(&self.storeroom_dir);
        let env_bytes = fs::metadata(&self.env_file).map(|m| m.len()).unwrap_or(0);

        let total_storage_bytes = cache_bytes + log_bytes + storeroom_bytes + env_bytes;

        FalzStats {
            root_path: self.root_dir.to_string_lossy().to_string(),
            cached_files_count,
            log_files_count,
            storeroom_slots_count,
            total_storage_bytes,
        }
    }

    /// Cleans up temporary cache and old logs
    pub fn clean(&self) -> std::io::Result<usize> {
        let mut cleaned = 0;
        if let Ok(entries) = fs::read_dir(&self.cache_dir) {
            for entry in entries.flatten() {
                if fs::remove_file(entry.path()).is_ok() {
                    cleaned += 1;
                }
            }
        }
        if let Ok(entries) = fs::read_dir(&self.logs_dir) {
            for entry in entries.flatten() {
                if fs::remove_file(entry.path()).is_ok() {
                    cleaned += 1;
                }
            }
        }
        Ok(cleaned)
    }
}
