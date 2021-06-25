#![no_std]
#![no_main]

use cortex_m::peripheral::Peripherals;
use panic_halt as _;
use smart_leds::{brightness, SmartLedsWrite, RGB8};
use stm32f4xx_hal::delay::Delay;
use stm32f4xx_hal::prelude::*;
use stm32f4xx_hal::spi::{NoMiso, NoSck, Spi};
use stm32f4xx_hal::stm32;
use ws2812_spi::Ws2812;

#[cortex_m_rt::entry]
fn main() -> ! {
    if let (Some(dp), Some(cp)) = (stm32::Peripherals::take(), Peripherals::take()) {
        let rcc = dp.RCC.constrain();
        let clocks = rcc.cfgr.use_hse(20.mhz()).sysclk(48.mhz()).freeze();
        let gpiob = dp.GPIOB.split();
        let led = gpiob.pb15.into_alternate_af5();
        let clk = gpiob.pb13.into_alternate_af5();
        let mut delay = Delay::new(cp.SYST, clocks);
        let spi = Spi::spi2(
            dp.SPI2,
            (clk, NoMiso, led),
            ws2812_spi::MODE,
            3_000_000.hz(),
            clocks,
        );
        let mut ws = Ws2812::new(spi);
        let mut data = [RGB8::default(); 12];

        const BRIGHTNESS: u8 = 255;

        loop {
            // Run through 25 colours (0 to 250 into the colour wheel).
            for i in 0..25 {
                let drop = wheel(i * 10);

                // Pulse the colour along the LED chain excluding the final LED.
                for led in 0..11 {
                    for j in 0..11 {
                        if j == led {
                            data[j] = drop;
                        } else {
                            data[j] = RGB8::default();
                        }
                    }
                    ws.write(brightness(data.iter().cloned(), BRIGHTNESS))
                        .unwrap();
                    delay.delay_ms(50u8);
                }

                // Now load the colour into the final LED.
                data[10] = RGB8::default();
                data[11] = drop;
                ws.write(brightness(data.iter().cloned(), BRIGHTNESS))
                    .unwrap();
                delay.delay_ms(50u8);

                // Fade out the colour in the final LED.
                for _ in 0..=255 {
                    data[11].r = data[11].r.saturating_sub(1);
                    data[11].g = data[11].g.saturating_sub(1);
                    data[11].b = data[11].b.saturating_sub(1);
                    ws.write(brightness(data.iter().cloned(), BRIGHTNESS))
                        .unwrap();
                    delay.delay_ms(1u8);
                }
            }
        }
    }
    loop {}
}

// Colour wheel from https://github.com/smart-leds-rs/smart-leds-samples
fn wheel(mut wheel_pos: u8) -> RGB8 {
    wheel_pos = 255 - wheel_pos;
    if wheel_pos < 85 {
        return (255 - wheel_pos * 3, 0, wheel_pos * 3).into();
    }
    if wheel_pos < 170 {
        wheel_pos -= 85;
        return (0, wheel_pos * 3, 255 - wheel_pos * 3).into();
    }
    wheel_pos -= 170;
    (wheel_pos * 3, 255 - wheel_pos * 3, 0).into()
}
