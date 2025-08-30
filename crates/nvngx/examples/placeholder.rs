// Minimal placeholder example for the nvngx crate.
// Prints the Vulkan instance and device extensions NGX requires.
// Note: running this binary may require the NGX runtime libraries to be discoverable by the dynamic linker.

fn main() {
    env_logger::init();

    println!("nvngx placeholder example\n---------------------------");
    match nvngx::vk::RequiredExtensions::get() {
        Ok(req) => {
            println!("Required instance extensions:");
            for e in &req.instance {
                println!("  - {}", e);
            }
            println!("\nRequired device extensions:");
            for e in &req.device {
                println!("  - {}", e);
            }
        }
        Err(e) => {
            eprintln!("Failed to query required extensions: {}", e);
            std::process::exit(1);
        }
    }
}
