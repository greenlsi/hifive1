#![no_main]
#![no_std]

/*
 Demonstration on how to use the feature "virq" from e310x-hal.
This feature enables a kind of vectorized interrupt matching for
all 52 the external interrupts that e310x has. It simply offers a convenient
way to handle each interrupt separately with a function called as the interrupt source.
For example, if an interrupt for GPIO0 is received, and a no mangled function called GPIO0()
exists, that function will automatically handle the exception, and it will be automatically
marked as complete by the PLIC.
This can be applied for all the 52 interrupts declared in e310x/interrupts.rs.
*/

extern crate panic_halt;

use hifive1::{hal::prelude::*, hal::DeviceResources, pin, sprintln};

use riscv::register::mstatus;
use riscv_rt::entry;

/* we have chosen the GPIO4 (a.k.a dig12) for this example */
// const GPIO_N: usize = 4;

/* Handler for the GPIO0 interrupt */
#[no_mangle]
#[allow(non_snake_case)]
fn RTC() {
    sprintln!("We reached the RTC interrupt!");
    unsafe {
        let mut rtc = hifive1::hal::DeviceResources::steal().peripherals.RTC.constrain();
        rtc.set_rtccmp(rtc.rtc_lo() + 10000);
    }
    /* Clear the GPIO pending interrupt */
    // unsafe {
    //     let gpio_block = &*hifive1::hal::e310x::GPIO0::ptr();
    //     gpio_block.fall_ip.write(|w| w.bits(1 << GPIO_N));
    // }
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

    /* Set GPIO4 (pin 12) as input */
    // let gpio4 = pin!(gpio, dig12);
    // let input = gpio.pin4.into_pull_up_input();
    // let input = gpio4.into_pull_up_input();

    /* Wrapper for easy access */
    let mut plic = resources.core_peripherals.plic;

     let mut rtc = peripherals.RTC.constrain();
    rtc.disable();
    rtc.set_scale(0);
    rtc.set_rtc(0);
    rtc.set_rtccmp(10000);
    rtc.enable();
    sprintln!("Init!");
    /* Unsafe block */
    unsafe {
        plic.reset();
        /* Get raw PLIC pointer */
        //let rplic = &*hifive1::hal::e310x::PLIC::ptr();
        plic.set_threshold(e310x_hal::e310x::Priority::P1);
        plic.enable_interrupt(hifive1::hal::e310x::Interrupt::RTC);
        hifive1::hal::e310x::PLIC::enable();
        /* Activate global interrupts (mie bit) */
        riscv::register::mie::set_mext();
        hifive1::hal::e310x::PLIC::set_priority(
            &mut plic,
            hifive1::hal::e310x::Interrupt::RTC,
            e310x_hal::e310x::Priority::P7,
        );
        mstatus::set_mie();
    }
    loop{}

}
