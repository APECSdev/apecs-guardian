// src/ui/welcome.rs

/*! Welcome — startup banner. Pure output, no logic. */

/// Print the startup banner.
pub fn banner(version: &str) {
    println!();
    println!("    ___    ____  _____________");
    println!("   /   |  / __ \\/ ____/ ___/  APECS Guardian");
    println!("  / /| | / /_/ / __/  \\__ \\   read-only node monitor");
    println!(" / ___ |/ ____/ /___ ___/ /   {}", version);
    println!("/_/  |_/_/   /_____//____/    https://apecs.dev");
    println!();
    println!("  Monitoring: dash, bitcoin, bitcoin cash, avalanche, dogecoin, litecoin");
    println!();
}
