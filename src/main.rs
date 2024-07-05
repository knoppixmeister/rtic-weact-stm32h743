// #![deny(warnings)]
#![deny(unsafe_code)]
#![no_std]
#![no_main]

use rtic::app;
use rtic_monotonics::systick::prelude::*;

// release profile
// #[cfg(not(debug_assertions))]
// https://droogmic.github.io/microrust/getting-started/01.00.BUILD.html#build-3
extern crate panic_halt;

// use panic_semihosting as _;

systick_monotonic!(Mono, 1000);

#[app(
  device = stm32h7xx_hal::stm32,
  peripherals = true,
  dispatchers = [SPI1, FLASH]
)]
mod app {
  use super::*; // looks like inject all imports from upper/parent namespace. Need for import/inject Mono object

  use stm32h7xx_hal::pac::USART1;
  use stm32h7xx_hal::{
    // pac,
    prelude::*,
    serial::Tx
  };

  // use cortex_m_semihosting::hprintln;

  use core::fmt::Write;

  use stm32h7xx_hal::gpio::gpioc::{PC13};
  use stm32h7xx_hal::gpio::gpioe::{PE3};
  use stm32h7xx_hal::gpio::{Edge, ExtiPin, Input};
  use stm32h7xx_hal::gpio::{Output, PushPull};

  /*
  use systick_monotonic::{
    fugit::{Duration, RateExtU32, TimerInstantU64},
    ExtU64, Systick,
  };
  */

  // A monotonic timer to enable scheduling in RTIC
  // #[monotonic(binds = SysTick, default = true)]
  // type MonotonicTick = Systick<500>; // 500 Hz / 2 ms granularity

  #[shared]
  struct SharedResources {
    tx: Tx<USART1>,
    led: PE3<Output<PushPull>>,
  }

  #[local]
  struct LocalResources {
    button: PC13<Input>,
    // led: PE3<Output<PushPull>>,
  }

  #[init]
  fn init(mut ctx: init::Context) -> (SharedResources, LocalResources) {
    let pwr = ctx.device.PWR.constrain();
    let pwrcfg = pwr.freeze();

    // Initialize the systick interrupt & obtain the token to prove that we did
    Mono::start(ctx.core.SYST, 400_000_000); // default STM32F303 clock-rate is 36MHz

    // RCC
    let rcc = ctx.device.RCC.constrain();
    let ccdr = rcc
                        .sys_ck(400.MHz())
                        .freeze(pwrcfg, &ctx.device.SYSCFG);

    // GPIO
    let gpioc = ctx.device.GPIOC.split(ccdr.peripheral.GPIOC);
    let gpiob = ctx.device.GPIOB.split(ccdr.peripheral.GPIOB);
    let gpioe = ctx.device.GPIOE.split(ccdr.peripheral.GPIOE);

    // https://github.com/mygnu/rregatta-firmware/blob/main/src/main.rs#L64
    let mut button = gpioc.pc13.into_pull_down_input(); // <!-- DON"t change it for ... float_input ...
    button.make_interrupt_source(&mut ctx.device.SYSCFG);
    button.trigger_on_edge(&mut ctx.device.EXTI, Edge::Rising);
    button.enable_interrupt(&mut ctx.device.EXTI);

    // local_print();
    // hprintln!("D - fsdfsdfsdfsdfsdf f dfdf");

    let tx = gpiob.pb14.into_alternate();
    let rx = gpiob.pb15.into_alternate();

    let serial = ctx.device
                                    .USART1
                                    .serial((tx, rx), 115200.bps(), ccdr.peripheral.USART1, &ccdr.clocks)
                                    .unwrap();

    let (mut tx, _) = serial.split();

    writeln!(tx, "Hello, world!").unwrap();

    let ld: PE3<Output<PushPull>> = gpioe.pe3.into_push_pull_output();

    blinking::spawn().ok();

    (
      SharedResources {
        tx,
        led: ld
      },
      LocalResources {
        button,
        // led: ld,
      }
    )
  }

  #[task(binds = EXTI15_10, local = [button], shared = [tx, led])]
  fn button_click(ctx: button_click::Context) {
    // hprintln!("btn");

    // ctx.local.led.toggle();

    foo::spawn().unwrap();

    if ctx.local.button.is_high() {
      // hprintln!("IS_H");
    }
    else {
      // hprintln!("IS_LOW");
    }

    ctx.local.button.clear_interrupt_pending_bit();
  }

  #[idle(local = [x: u32 = 0], shared = [tx])]
  fn idle(_cx: idle::Context) -> ! {
    // Locals in idle have lifetime 'static
    let _x: &'static mut u32 = _cx.local.x;

    // hprintln!("idle: {}", _x);

    // debug::exit(debug::EXIT_SUCCESS); // Exit QEMU simulator

    loop {
      cortex_m::asm::nop();
    }
  }

  #[task(priority = 1, shared=[tx])]
  async fn foo(mut cx: foo::Context) {
    // local_print();

    cx.shared.tx.lock(|tx| {
      writeln!(tx, "Fofoofofof").unwrap();
    });

    // hprintln!("Fooooo odofodfodofdofodfo");
  }

  #[task(priority = 1, shared=[tx, led])]
  async fn blinking(mut ctx: blinking::Context) {
    loop {
      ctx.shared.led.lock(|led| {
        led.toggle();
      });

      ctx.shared.tx.lock(|tx| {
        writeln!(tx, "Blink ....").unwrap();
      });

      Mono::delay(1000.millis()).await;
    }
  }

  #[task(priority = 3, shared=[tx])]
  async fn printing(mut ctx: printing::Context) {
    loop {
      ctx.shared.tx.lock(|tx| {
        writeln!(tx, "Printing ....").unwrap();
      });

      Mono::delay((5*1000).millis()).await;
    }
  }
  
}
