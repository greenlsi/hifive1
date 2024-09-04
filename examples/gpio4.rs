//! Demonstration on how to configure the GPIO4 interrupt on HiFive boards.
//! You must connect a button to pin 12 (GPIO4) and ground to test this example.

#![no_main]
#![no_std]

extern crate panic_halt;

use hifive1::hal::e310x::PLIC;
use hifive1::{hal::prelude::*, hal::DeviceResources, pin, sprintln};

use riscv::register::mstatus;
use riscv_rt::entry;

/* Handler for the GPIO0 interrupt */
#[riscv_rt::external_interrupt(ExternalInterrupt::GPIO4)]
fn gpio4_handler() {
    sprintln!("We reached the GPIO4 interrupt!");
    /* Clear the GPIO pending interrupt */
    let gpio_block = unsafe { hifive1::hal::e310x::Gpio0::steal() };
    gpio_block.fall_ip().write(|w| w.pin4().set_bit());
}

/* Code adapted from https://github.com/riscv-rust/riscv-rust-quickstart/blob/interrupt-test/examples/interrupt.rs*/
#[entry]
fn main() -> ! {
    /* Get the ownership of the device resources singleton */
    let resources = DeviceResources::take().unwrap();
    let peripherals = resources.peripherals;

    /* Configure system clock */
    let sysclock = hifive1::configure_clocks(peripherals.PRCI, peripherals.AONCLK, 64.mhz().into());
    /* Get the board pins */
    let gpio = resources.pins;

    /* Configure stdout for debugging */
    hifive1::stdout::configure(
        peripherals.UART0,
        pin!(gpio, uart0_tx),
        pin!(gpio, uart0_rx),
        115_200.bps(),
        sysclock,
    );

    sprintln!("Configuring GPIO...");
    /* Set GPIO4 (pin 12) as input */
    // let gpio4 = pin!(gpio, dig12);
    gpio.pin4.into_pull_up_input();
    //let input = gpio4.into_pull_up_input();

    sprintln!("Configuring priorities...");
    /* Set interrupt source priority */
    let priorities = PLIC::priorities();
    unsafe { priorities.set_priority(ExternalInterrupt::GPIO4, Priority::P7) };

    let gpio_block = unsafe { hifive1::hal::e310x::Gpio0::steal() };
    unsafe {
        /* Clear pending interrupts from previous states */
        gpio_block.fall_ie().write(|w| w.bits(0x00000000));
        gpio_block.rise_ie().write(|w| w.bits(0x00000000));
        gpio_block.fall_ip().write(|w| w.bits(0xffffffff));
        gpio_block.rise_ip().write(|w| w.bits(0xffffffff));
    }
    gpio_block.fall_ie().write(|w| w.pin4().set_bit());
    gpio_block.rise_ie().write(|w| w.pin4().clear_bit());

    /* Activate global interrupts (mie bit) */
    let ctx = PLIC::ctx0();
    unsafe {
        ctx.threshold().set_threshold(Priority::P1);
        ctx.enables().enable(ExternalInterrupt::GPIO4);
        mstatus::set_mie();
        PLIC::enable();
    }
    loop {
        riscv::asm::wfi();
    }
}
