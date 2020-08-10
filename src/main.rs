#![feature(asm)]
#![no_main]
#![no_std]

mod game_fs;
mod pksm;
mod utils;

use core::panic::PanicInfo;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    loop {}
}

static EXTDATA_BANK_FILE_PATH: &str = "/banks/transport.bnk\0";
static SD_BANK_FILE_PATH: &str = "/3ds/PKSM/banks/transport.bnk\0";

// Instead of reading PKSM's config JSON file, we'll just read each location and default to extdata
fn get_bank() -> pksm::Bank {
    let mut bank = pksm::Bank::new(EXTDATA_BANK_FILE_PATH, true);

    if !bank.is_valid() {
        bank.close();
        bank = pksm::Bank::new(SD_BANK_FILE_PATH, false);
    }

    return bank;
}

fn send_pokemon_to_bank() -> u32 {
    let mut bank = get_bank();

    bank.transfer_box();

    bank.close();

    return 0x10;
}

fn verify_safe_transfer() -> u32 {
    let mut bank = get_bank();

    let is_valid = bank.is_valid() && bank.is_first_box_empty();

    let next_state = if is_valid { 3 } else { 7 };

    bank.close();

    return next_state;
}

// The make-ips script relies on a single entrypoint for ease of locating it in the built binary
// Since there are two functions we want to run, we need to check the return address and route the patch appropriately
// Transporter is a giant state machine, and each state sets the next state.  As a result, each patch function returns the next state
#[no_mangle]
pub extern "C" fn _start() -> u32 {
    let return_addr: u32;

    unsafe {
        asm!("mov {}, lr", out(reg) return_addr);
    }

    if return_addr == 0x24a3dc {
        return send_pokemon_to_bank();
    } else if return_addr == 0x248d40 {
        return verify_safe_transfer(); // This returns the next state
    }

    return 0;
}
