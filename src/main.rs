use win_ring0::WinRing0;
 
pub fn main() {
    let mut r0: Box<WinRing0> = Box::from(WinRing0::new());
 
    println!("Installing ring0 driver");
    match r0.install() {
        Ok(()) => { println!("Driver installed"); }
        Err(err) => { println!("Error: {}", err); }
    }
 
    println!("Opening ring0 driver");
    match r0.open() {
        Ok(()) => { println!("Driver opened"); }
        Err(err) => { println!("Error: {}", err); }
    }
 
    println!("Trying to get tjMax value, should work on most Intel CPUs");
    // MSR_TEMPERATURE_TARGET
    let msr = 0x1a2;
    let out = r0.readMsr(msr).unwrap();
 
    let _edx = ((out >> 32) & 0xffffffff) as u32;
    let eax = (out & 0xffffffff) as u32;
    let tj_max = (eax >> 16) & 0xff;

    
    // MSR_RAPL_POWER_UNIT
    let msr = 0x606;
    // The 64 bit value returned into EDX:EAX registers
    let out = r0.readMsr(msr).unwrap();
    // Split the 64 bit value into the original by offsetting and masking
    // EDX: high order 32 bits
    let edx = ((out >> 32) & 0xffffffff) as u32;
    // EAX: low order 32 bits
    let eax = (out & 0xffffffff) as u32;
    // power_unit = 1/2^PU where PU is bits 3:0 of EAX
    let power_unit = 1.0 / f64::from(1 << (eax & 0xf));
    // energy_unit = 1/2^EU where EU is bits 12:8 of EAX
    let energy_unit = 1.0 / f64::from(1 << ((eax >> 8) & 0x1f));
    // time_unit = 1/2^TU where TU is bits 19:16 of EAX
    let time_unit = 1.0 / f64::from(1 << ((eax >> 16) & 0xf));
    
    println!("Raw Power Unit: {:b}", eax & 0xf);
    println!("Raw Energy Unit: {:b}", (eax >> 8) & 0x1f);
    println!("Raw Time Unit: {:b}", (eax >> 16) & 0xf);

    println!("Power Unit: {}mW", power_unit);
    println!("Energy Unit: {}µJ", energy_unit);
    println!("Time Unit: {}s", time_unit);

    // let mut count = 0;
    // while count < 100 {
    //     // MSR_PKG_ENERGY_STATUS
    //     let msr = 0x611;
    //     let out = r0.readMsr(msr).unwrap();
    //     let edx = ((out >> 32) & 0xffffffff) as u32;
    //     let eax = (out & 0xffffffff) as u32;
    //     let energy = ((edx as u64) << 32) | (eax as u64);
    //     println!("Energy: {}", energy);
    //     count += 1;
    // }  
 
    println!("MSR Value: {}", tj_max);
 
    println!("Closing ring0 driver");
    match r0.close() {
        Ok(()) => { println!("Driver closed"); }
        Err(err) => { println!("Error: {}", err); }
    }
 
    println!("Uninstall ring0 driver");
    match r0.uninstall() {
        Ok(()) => { println!("Driver uninstalled"); }
        Err(err) => { println!("Error: {}", err); }
    }    
}