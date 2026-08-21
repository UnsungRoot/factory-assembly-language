use crate::falz::FalzEnvironment;
use crate::jit::DynamicJitEngine;
use crate::parser::Parser;
use crate::target::TargetContext;
use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::Path;

pub struct Cli;

impl Cli {
    pub fn run(args: Vec<String>) {
        if args.len() < 2 {
            Self::print_help();
            return;
        }

        let cmd = args[1].to_lowercase();
        match cmd.as_str() {
            "help" | "--help" | "-h" => Self::print_help(),
            "env" | "doctor" => Self::cmd_env(),
            "clean" => Self::cmd_clean(),
            "new" => {
                if args.len() < 3 {
                    eprintln!("Error: Missing project name.\nUsage: fal new <project_name>");
                    std::process::exit(1);
                }
                Self::cmd_new(&args[2]);
            }
            "debug" => {
                if args.len() < 3 {
                    eprintln!("Error: Missing .fal file.\nUsage: fal debug <file.fal>");
                    std::process::exit(1);
                }
                Self::cmd_run(&args[2], true);
            }
            "run" => {
                if args.len() < 3 {
                    eprintln!("Error: Missing .fal file.\nUsage: fal run <file.fal>");
                    std::process::exit(1);
                }
                Self::cmd_run(&args[2], false);
            }
            // If the argument is a .fal file path directly: `fal program.fal`
            file_path if file_path.ends_with(".fal") || Path::new(file_path).exists() => {
                Self::cmd_run(file_path, false);
            }
            unknown => {
                eprintln!("Error: Unknown command '{}'", unknown);
                Self::print_help();
                std::process::exit(1);
            }
        }
    }

    fn cmd_run(file_path: &str, debug: bool) {
        let target_ctx = TargetContext::autodetect();
        let falz = FalzEnvironment::init(&target_ctx);

        let file = match File::open(file_path) {
            Ok(f) => f,
            Err(e) => {
                eprintln!("Error: Failed to open FAL file '{}': {}", file_path, e);
                std::process::exit(1);
            }
        };

        let reader = BufReader::new(file);
        let mut jit = DynamicJitEngine::new();
        let mut line_count = 0;

        for line in reader.lines() {
            if let Ok(line_content) = line {
                line_count += 1;
                if let Some(instruction) = Parser::parse_line(&line_content) {
                    jit.compile_instruction(&instruction);
                }
            }
        }

        let result = jit.execute(debug);

        let log_summary = format!(
            "Program: {}\nLines Processed: {}\nPrimary Tray Result: {}\nFALZ Home: {}\n",
            file_path, line_count, result, falz.root_dir.display()
        );
        falz.save_log(file_path, &log_summary);

        if debug {
            println!("=== [FAL VISUAL DEBUGGER LOG] ===");
            println!("Primary Tray Result : {}", result);
            println!("FALZ Session Log    : Saved in {}", falz.logs_dir.display());
            println!("=================================\n");
        }
    }

    fn cmd_new(project_name: &str) {
        let proj_path = Path::new(project_name);
        if proj_path.exists() {
            eprintln!("Error: Directory '{}' already exists!", project_name);
            std::process::exit(1);
        }

        if let Err(e) = fs::create_dir_all(proj_path) {
            eprintln!("Error creating project directory: {}", e);
            std::process::exit(1);
        }

        let factory_code = r#"// factory.fal - Starter Factory Program
workstation "main":
    say "[FAL] Starting factory floor..."

    // Pillar 1 & 2: Trays & Math
    tray1 = 100
    tray2 = 50
    add tray2 to tray1

    // Pillar 4: Storeroom (RAM)
    store tray1 into storeroom[0]

    // Pillar 3: Workstation Routing
    call workstation "inspect_quality"

    say "[FAL] Factory shift completed successfully!"
    call_supervisor exit
end

workstation "inspect_quality":
    load storeroom[0] into tray3
    if tray3 == 150 then tray4 = "Quality Check: 100% Passed"
    say tray4
    return tray3
end
"#;

        let readme_content = format!(
            "# {}\n\nA Factory Assembly Language (FAL) project.\n\n## Run\n```bash\nfal run factory.fal\n```\n",
            project_name
        );

        let _ = fs::write(proj_path.join("factory.fal"), factory_code);
        let _ = fs::write(proj_path.join("README.md"), readme_content);

        println!("Created new FAL project '{}'!", project_name);
        println!("   Location : {}", proj_path.display());
        println!("   Entry    : {}/factory.fal", project_name);
        println!("\nTo run your new project:");
        println!("   cd {}", project_name);
        println!("   fal run factory.fal\n");
    }

    fn cmd_env() {
        let target_ctx = TargetContext::autodetect();
        let falz = FalzEnvironment::init(&target_ctx);
        let stats = falz.get_stats();

        println!("=== FAL & FALZ ENVIRONMENT REPORT ===");
        println!("Target CPU Architecture  : {:?}", target_ctx.arch);
        println!("Host OS Supervisor       : {:?}", target_ctx.os);
        println!("Physical Trays Detected  : {}", target_ctx.max_physical_trays);
        println!("SSL Register Pool        : {:?}", target_ctx.available_registers);
        println!("----------------------------------------");
        println!("FALZ Global Storage Path : {}", stats.root_path);
        println!("FALZ Hardware Profile    : {}/env.json", stats.root_path);
        println!("FALZ Cached Artifacts    : {} files", stats.cached_files_count);
        println!("FALZ Storeroom Slots     : {} allocated", stats.storeroom_slots_count);
        println!("FALZ Session Logs        : {} files", stats.log_files_count);
        println!("FALZ Total Storage Size  : {} bytes", stats.total_storage_bytes);
        println!("========================================\n");
    }


    fn cmd_clean() {
        let target_ctx = TargetContext::autodetect();
        let falz = FalzEnvironment::init(&target_ctx);
        match falz.clean() {
            Ok(count) => println!("Cleaned FALZ storage: removed {} cached/log files.", count),
            Err(e) => eprintln!("Error cleaning FALZ storage: {}", e),
        }
    }

    fn print_help() {
        println!("Factory Assembly Language (FAL) CLI");
        println!("   High-performance native JIT compiler with FALZ global storage.\n");
        println!("USAGE:");
        println!("   fal <COMMAND> [OPTIONS]\n");
        println!("COMMANDS:");
        println!("   run <file.fal>       Compile and execute a .fal program on the spot (JIT)");
        println!("   debug <file.fal>     Execute with step-by-step visual debugger logs");
        println!("   new <project_name>   Scaffold a new FAL factory project");
        println!("   env / doctor         Display host hardware detection & FALZ storage status");
        println!("   clean                Clean up FALZ global cache and old execution logs");
        println!("   help                 Print this help menu\n");
        println!("EXAMPLES:");
        println!("   fal run examples/strings_and_conditions.fal");
        println!("   fal new my_factory_app");
        println!("   fal env\n");
    }
}

