#![no_main]
#![no_std]

/*
 Demonstration on how to use the feature "v-extern" from e310x-hal.
This feature enables a kind of vectorized interrupt matching for
all 52 the external interrupts that e310x has. It simply offers a convenient
way to handle each interrupt separately with a function called as the interrupt source.
For example, if an interrupt for GPIO0 is received, and a no mangled function called GPIO0()
exists, that function will automatically handle the exception, and it will be automatically
marked as complete by the PLIC.
This can be applied for all the 52 interrupts declared in e310x/interrupts.rs.
*/

extern crate panic_halt;

use e310x_hal::e310x::{Interrupt, Priority, PLIC};
use hifive1::{hal::prelude::*, hal::DeviceResources, pin, sprintln};
use riscv_rt::entry;

/* Handler for the RTC interrupt */
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
}

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

    sprintln!("Configuring PLIC...");
    // First, we make sure that all PLIC the interrupts are disabled and set the interrupts priorities
    PLIC::disable();
    PLIC::priorities().reset::<Interrupt>();
    // Safety: interrupts are disabled
    unsafe { PLIC::priorities().set_priority(Interrupt::RTC, Priority::P7) };

    // Next, we configure the PLIC context for our use case
    let ctx = PLIC::ctx0();
    ctx.enables().disable_all::<Interrupt>();
    // Safety: we are the only hart running and we have not enabled any interrupts yet
    unsafe {
        ctx.enables().enable(Interrupt::RTC);
        ctx.threshold().set_threshold(Priority::P1);
    };
    sprintln!("done!");

    sprintln!("Configuring RTC...");
    let mut rtc = peripherals.RTC.constrain();
    rtc.disable();
    rtc.set_scale(0);
    rtc.set_rtc(0);
    rtc.set_rtccmp(10000);
    rtc.enable();
    sprintln!("done!");

    sprintln!("Enabling interrupts...");
    unsafe {
        riscv::interrupt::enable();
        PLIC::enable();
    }
    loop {
        sprintln!("Going to sleep!");
        unsafe { riscv::asm::wfi() };
    }
}
