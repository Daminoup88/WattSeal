use std::{any::Any, ffi::c_ulong, panic, process::Command};

use win_ring0::WinRing0;

pub struct WinRing0Reader {
    ring0: WinRing0,
}

impl WinRing0Reader {
    pub fn new() -> Result<Self, String> {
        println!("Initializing WinRing0 driver...");
        let mut handler = panic::catch_unwind(|| WinRing0Reader { ring0: WinRing0::new() })
            .unwrap_or_else(WinRing0Reader::free_stuck_driver);
        println!("Installing WinRing0 driver...");
        handler.ring0.install()?;
        println!("Opening WinRing0 driver...");
        handler.ring0.open()?;
        println!("WinRing0 driver opened successfully.");
        Ok(handler)
    }

    pub fn read_msr(&self, msr: c_ulong) -> Result<u64, String> {
        self.ring0.readMsr(msr)
    }

    fn free_stuck_driver(_err: Box<dyn Any + Send>) -> Self {
        println!("WinRing0 initialization panicked. Freeing stuck driver...");
        // sc stop WinRing0_1_2_0
        Command::new("sc")
            .args(["stop", "WinRing0_1_2_0"])
            .output()
            .expect("sc stop failed");
        println!("Stuck WinRing0 driver stopped successfully.");
        let mut handler = WinRing0Reader { ring0: WinRing0::new() };
        match handler.ring0.uninstall() {
            Ok(_) => println!("Stuck WinRing0 driver uninstalled successfully."),
            Err(e) => println!("Error uninstalling stuck WinRing0 driver: {}", e),
        }
        return handler;
    }
}

impl Drop for WinRing0Reader {
    fn drop(&mut self) {
        println!("Closing WinRing0 driver...");
        match self.ring0.close() {
            Ok(_) => println!("WinRing0 driver closed successfully."),
            Err(e) => println!("Error closing WinRing0 driver: {}", e),
        }
        println!("Uninstalling WinRing0 driver...");
        match self.ring0.uninstall() {
            Ok(_) => println!("WinRing0 driver uninstalled successfully."),
            Err(e) => println!("Error uninstalling WinRing0 driver: {}", e),
        }
    }
}
