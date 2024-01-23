#![no_std]
#![no_main]

use core::panic::PanicInfo;
mod asm;
mod reader;

use asm::writeln;
use reader::MReader;

#[no_mangle]
pub extern "C" fn main(_argc: isize, _argv: *const *const u8) -> isize {
    let lines = MReader::new("Cargo.toml\0");
    for line in lines.into_iter() {
        writeln(line);
    }

    0
}

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}
