use sp1_build::{build_program_with_args, BuildArgs};

fn main() {
    // Ensure output directory exists
    std::fs::create_dir_all("elf").unwrap();
    
    let args = BuildArgs {
        elf_name: Some("bridge-constraints-elf".to_string()),
        output_directory: Some("elf".to_string()),
        locked: true,
        ..Default::default()
    };
    
    build_program_with_args("../bridge-constraints-sp1-program", args);
} 