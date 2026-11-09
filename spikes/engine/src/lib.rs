// SPDX-License-Identifier: RPL-1.5
//! G3 (P0-C) throwaway spikes. One binary per spike in src/bin, each is its own check (exit 0 = pass).
pub use gdm_fixture as fixture; // moved to fixtures/hostile (G5)
pub mod seg;

/// assert that prints a PASS/FAIL line and counts failures
pub struct Check { pub fails: u32, pub name: &'static str }
impl Check {
    pub fn new(name: &'static str) -> Self { println!("== {name}"); Check { fails: 0, name } }
    pub fn ok(&mut self, cond: bool, what: &str) {
        println!("{} {what}", if cond { "PASS" } else { "FAIL" });
        if !cond { self.fails += 1; }
    }
    pub fn note(&self, what: &str) { println!("NOTE {what}"); }
    pub fn finish(self) -> ! {
        println!("== {}: {}", self.name, if self.fails == 0 { "ALL PASS".to_string() } else { format!("{} FAIL", self.fails) });
        std::process::exit(if self.fails == 0 { 0 } else { 1 })
    }
}
