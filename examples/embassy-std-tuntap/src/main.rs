use std::net::Ipv6Addr;
use std::str::FromStr;

use clap::Parser;
use embassy_executor::{Executor, Spawner};
use embassy_net::udp::{PacketMetadata, UdpSocket};
use embassy_net::{Config, Ipv6Cidr, StackResources};
use embassy_net_tuntap::TunTapDevice;
use embassy_net::StaticConfigV6;
use log::*;
use rand_core::{OsRng, RngCore};
use static_cell::StaticCell;

#[derive(Parser)]
#[clap(version = "1.0")]
struct Opts {
    #[clap(long, default_value = "tap99")]
    tap: String,
    #[clap(long, default_value = "0")]
    static_id: u8,
}

#[embassy_executor::task]
async fn net_task(mut runner: embassy_net::Runner<'static, TunTapDevice>) -> ! {
    runner.run().await
}

#[embassy_executor::task]
async fn main_task(spawner: Spawner) {
    let opts: Opts = Opts::parse();

    // Init network device
    let device = TunTapDevice::new(&opts.tap).unwrap();

    // Generate random seed
    let mut seed = [0; 8];
    OsRng.fill_bytes(&mut seed);

    // use first byte of seed as the last ipv6 addr byte, or the one from clap if provided.
    let address_byte : u8 = if opts.static_id == 0 {
        seed[0] as u8
    } else {
        opts.static_id
    };

    let rand_addr = Ipv6Addr::from_str(&format!("fe80::{}",address_byte)).unwrap();

    info!("Address: {}",rand_addr);
    let seed = u64::from_le_bytes(seed);

    let config = Config::ipv6_static(StaticConfigV6 {
        address: Ipv6Cidr::new(rand_addr, 64),
        gateway: None, // Some(Ipv6Addr::from_str("fe80::1").unwrap()),
        dns_servers: heapless::Vec::new(),
    });

    // Init network stack
    static RESOURCES: StaticCell<StackResources<3>> = StaticCell::new();
    let (stack, runner) = embassy_net::new(device, config, RESOURCES.init(StackResources::new()), seed);

    // Launch network task
    spawner.spawn(net_task(runner)).unwrap();

    // Then we can use it!
    let mut rx_meta = [PacketMetadata::EMPTY; 16];
    let mut rx_buffer = [0; 4096];
    let mut tx_meta = [PacketMetadata::EMPTY; 16];
    let mut tx_buffer = [0; 4096];
    let mut buf = [0; 4096];

    let mut socket = UdpSocket::new(stack, &mut rx_meta, &mut rx_buffer, &mut tx_meta, &mut tx_buffer);
    socket.bind(9400).unwrap();

    loop {
        let (n, ep) = socket.recv_from(&mut buf).await.unwrap();
        if let Ok(s) = core::str::from_utf8(&buf[..n]) {
            info!("ECHO (to {}): {}", ep, s);
        } else {
            info!("ECHO (to {}): bytearray len {}", ep, n);
        }
        socket.send_to(&buf[..n], ep).await.unwrap();
    }
}

static EXECUTOR: StaticCell<Executor> = StaticCell::new();

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .filter_module("async_io", log::LevelFilter::Debug)
        .format_timestamp_nanos()
        .init();

    let executor = EXECUTOR.init(Executor::new());
    executor.run(|spawner| {
        spawner.spawn(main_task(spawner)).unwrap();
    });
}

