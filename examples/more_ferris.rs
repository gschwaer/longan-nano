#![no_std]
#![no_main]

use embedded_graphics::image::Image;
use embedded_graphics::image::ImageRaw;
use embedded_graphics::pixelcolor::raw::LittleEndian;
use embedded_sdmmc::VolumeIdx;
use longan_nano::sdcard;
use longan_nano::sdcard_pins;
use longan_nano::sprint;
use panic_halt as _;

use embedded_graphics::pixelcolor::Rgb565;
use embedded_graphics::prelude::*;
use embedded_graphics::primitive_style;
use embedded_graphics::primitives::Rectangle;
use longan_nano::hal::{pac, prelude::*};
use longan_nano::{lcd, lcd_pins, sprintln};
use riscv_rt::entry;

const IMAGE_WIDTH: u32 = 160;
const IMAGE_HEIGHT: u32 = 80;
const IMAGE_NUM_PIXELS: u32 = IMAGE_WIDTH * IMAGE_HEIGHT;
const IMAGE_NUM_BYTE: usize = (IMAGE_NUM_PIXELS * 2) as usize;

#[entry]
fn main() -> ! {
    let dp = pac::Peripherals::take().unwrap();

    // Configure clocks
    let mut rcu = dp.RCU.configure()
        .ext_hf_clock(8.mhz())
        .sysclk(108.mhz())
        .freeze();

    let mut afio = dp.AFIO.constrain(&mut rcu);

    let gpioa = dp.GPIOA.split(&mut rcu);
    let gpiob = dp.GPIOB.split(&mut rcu);

    // init serial
    longan_nano::stdout::configure(dp.USART0, gpioa.pa9, gpioa.pa10, 115_200.bps(), &mut afio, &mut rcu);

    // init lcd
    let lcd_pins = lcd_pins!(gpioa, gpiob);
    let mut lcd = lcd::configure(dp.SPI0, lcd_pins, &mut afio, &mut rcu);
    let (width, height) = (lcd.size().width as i32, lcd.size().height as i32);

    // Clear screen
    Rectangle::new(Point::new(0, 0), Point::new(width - 1, height - 1))
        .into_styled(primitive_style!(fill_color = Rgb565::BLACK))
        .draw(&mut lcd)
        .unwrap();

    // init sdcard
    let sdcard_pins = sdcard_pins!(gpiob);
    let mut sdcard = sdcard::configure(dp.SPI1, sdcard_pins, sdcard::SdCardFreq::Fast, &mut rcu);

    sprint!("Initializing sdcard: ");
    if let Err(_) = sdcard.device().init() {
        sprintln!("failed.");
    } else {
        sprintln!("ok.");
        // open first partition
        let mut volume = sdcard.get_volume(VolumeIdx(0)).unwrap();
        let root_dir = sdcard.open_root_dir(&volume).unwrap();

        sprint!("Looking for raw image sequence: ");
        if let Ok(_) = sdcard.find_directory_entry(&volume, &root_dir, "MORE_F~1.RAW") {
            sprintln!("found.");
            let mut buffer: [u8; IMAGE_NUM_BYTE] = [0; IMAGE_NUM_BYTE];
            let mut file = sdcard.open_file_in_dir(&mut volume, &root_dir, "MORE_F~1.RAW", embedded_sdmmc::Mode::ReadOnly).unwrap();
            loop {
                sdcard.read(&mut volume, &mut file, &mut buffer).unwrap();
                let raw_image: ImageRaw<Rgb565, LittleEndian> = ImageRaw::new(&buffer, IMAGE_WIDTH, IMAGE_HEIGHT);
                Image::new(&raw_image, Point::zero())
                    .draw(&mut lcd)
                    .unwrap();
                if file.eof() {
                    file.seek_from_start(0).unwrap();
                }
            }
        } else {
            sprintln!("not found.");
        }
    }

    loop {}
}
