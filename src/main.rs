// #![deny(warnings)]
#![deny(unsafe_code)]
#![no_std]
#![no_main]

// #[macro_use]
// mod utilities;

// release profile
// #[cfg(not(debug_assertions))]
// https://droogmic.github.io/microrust/getting-started/01.00.BUILD.html#build-3
extern crate panic_halt;

#[rtic::app(device = stm32h7xx_hal::stm32, peripherals = true)]
mod app {
    use stm32h7xx_hal::gpio::gpioc::{PC13};
    use stm32h7xx_hal::gpio::gpioe::{PE3};
    use stm32h7xx_hal::gpio::{Edge, ExtiPin, Input};
    use stm32h7xx_hal::gpio::{Output, PushPull};
    use stm32h7xx_hal::prelude::*;

    #[shared]
    struct SharedResources {
    }

    #[local]
    struct LocalResources {
      button: PC13<Input>,
      led: PE3<Output<PushPull>>,
    }

    #[init]
    fn init(
      mut ctx: init::Context,
    ) -> (SharedResources, LocalResources)
    {
      let pwr = ctx.device.PWR.constrain();
      let pwrcfg = pwr.freeze();

      // RCC
      let rcc = ctx.device.RCC.constrain();
      let ccdr = rcc
                          // .sys_ck(100.MHz())
                          .freeze(pwrcfg, &ctx.device.SYSCFG);

      // GPIO
      let gpioc = ctx.device.GPIOC.split(ccdr.peripheral.GPIOC);

      let gpioe = ctx.device.GPIOE.split(ccdr.peripheral.GPIOE);

      let mut button = gpioc.pc13.into_floating_input();
      button.make_interrupt_source(&mut ctx.device.SYSCFG);
      button.trigger_on_edge(&mut ctx.device.EXTI, Edge::Rising);
      button.enable_interrupt(&mut ctx.device.EXTI);

      (
        SharedResources {
        },
        LocalResources {
          button,
          led: gpioe.pe3.into_push_pull_output(),
        }
      )
    }

    #[task(binds = EXTI15_10, local = [button, led])]
    fn button_click(ctx: button_click::Context) {
      ctx.local.button.clear_interrupt_pending_bit();
      ctx.local.led.toggle();
    }

    #[idle(local = [x: u32 = 0])]
    fn idle(cx: idle::Context) -> ! {
        // Locals in idle have lifetime 'static
        // let _x: &'static mut u32 = cx.local.x;

        // hprintln!("idle");

        // debug::exit(debug::EXIT_SUCCESS); // Exit QEMU simulator

        loop {
          cortex_m::asm::nop();
        }
    }

    #[task(priority = 1)]
    async fn foo(_: foo::Context) {
    }
}
