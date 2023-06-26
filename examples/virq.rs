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

use e310x_hal::e310x::Priority;
use hifive1::{hal::prelude::*, hal::DeviceResources, pin, sprintln};

use riscv::register::mstatus;
use riscv_rt::entry;

/* we have chosen the GPIO4 (a.k.a dig12) for this example */
// const GPIO_N: usize = 4;

/* Handler for the GPIO0 interrupt */
#[no_mangle]
#[allow(non_snake_case)]
unsafe fn RTC() {
    sprintln!("-------------------");
    sprintln!("!start RTC");
    // increase rtccmp to clear HW interrupt
    let rtc = DeviceResources::steal().peripherals.RTC;
    let rtccmp = rtc.rtccmp.read().bits();
    rtc.rtccmp.write(|w| w.bits(rtccmp + 65536 * 2));
    sprintln!("!stop RTC (rtccmp = {})", rtccmp);
    sprintln!("-------------------");
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

    // Disable watchdog
    let wdg = peripherals.WDOG;
    wdg.wdogcfg.modify(|_, w| w.enalways().clear_bit());

    /* Set GPIO4 (pin 12) as input */
    // let gpio4 = pin!(gpio, dig12);
    // let input = gpio.pin4.into_pull_up_input();
    // let input = gpio4.into_pull_up_input();

    /* Wrapper for easy access */
    let mut plic = resources.core_peripherals.plic;

   
    sprintln!("Init!");
    /* Unsafe block */
    unsafe {
        plic.reset();
        /* Get raw PLIC pointer */
        plic.enable_interrupt(hifive1::hal::e310x::Interrupt::RTC);
        plic.set_priority(hifive1::hal::e310x::Interrupt::RTC, Priority::P7);
        plic.set_threshold(e310x_hal::e310x::Priority::P1);
    }
    sprintln!("done!");
    let mut rtc = peripherals.RTC.constrain();
    rtc.disable();
    rtc.set_scale(0);
    rtc.set_rtc(0);
    rtc.set_rtccmp(10000);
    rtc.enable();

    sprintln!("done!");
    unsafe {
        e310x::PLIC::enable();
        // mstatus::set_mie();
        riscv::interrupt::enable();
        // hifive1::hal::e310x::PLIC::enable();
    }

    sprintln!("done!");
    loop{}

}
