//! This example demonstrates how to configure the CLINT to generate
//! periodic interrupts using the machine timer.

#![no_main]
#![no_std]

extern crate panic_halt;

use hifive1::{
    configure_clocks,
    hal::{e310x::CLINT, prelude::*, DeviceResources},
    pin, sprintln,
};

const PERIOD_MS: u64 = 1000;
const FREQUENCY_HZ: u64 = 32768;
const CLINT_TICKS_PER_MS: u64 = PERIOD_MS * FREQUENCY_HZ / 1000;

/// Handler for the machine timer interrupt (handled by the CLINT)
#[riscv_rt::core_interrupt(CoreInterrupt::MachineTimer)]
fn mtimer_handler() {
    sprintln!("MTIMER interrupt!");
    CLINT::mtimecmp0().modify(|f| *f += CLINT_TICKS_PER_MS);
}

#[riscv_rt::entry]
fn main() -> ! {
    /* Get the ownership of the device resources singleton */
    let resources = DeviceResources::take().unwrap();
    let peripherals = resources.peripherals;

    /* Configure system clock */
    let sysclock = configure_clocks(peripherals.PRCI, peripherals.AONCLK, 64.mhz().into());

    /* Configure stdout for printing via UART */
    let gpio = resources.pins;
    hifive1::stdout::configure(
        peripherals.UART0,
        pin!(gpio, uart0_tx),
        pin!(gpio, uart0_rx),
        115_200.bps(),
        sysclock,
    );

    sprintln!("Configuring CLINT...");
    CLINT::mtimer_disable();
    let mtimer = CLINT::mtimer();
    let (mtimecmp, mtime) = (mtimer.mtimecmp0, mtimer.mtime);
    mtime.write(0);
    mtimecmp.write(CLINT_TICKS_PER_MS);

    sprintln!("Enabling interrupts...");
    unsafe {
        riscv::interrupt::enable();
        CLINT::mtimer_enable();
    }
    loop {
        sprintln!("Sleeping...");
        riscv::asm::wfi();
    }
}
