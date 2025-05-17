use embassy_executor::Executor;
use embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex;
use embassy_sync::channel::Channel;
use embassy_time::Timer;

use root::concepts::neighbour::Neighbour;
use root::concepts::packet::OutboundPacket;
use root::concepts::route::Route;
use root::framework::RoutingSystem;
use root::router::{NoMACSystem, Router};
use static_cell::StaticCell;

#[derive(Debug, Clone)]
struct SimpleExample {} // just a type to inform root of your network parameters

impl RoutingSystem for SimpleExample {
    type NodeAddress = String; // our nodes have string names
    type Link = i32;
    type MACSystem = NoMACSystem; // we won't use MAC for this example
}

static RADIOWAVES: Channel<CriticalSectionRawMutex, OutboundPacket<SimpleExample>, 8> =
    Channel::new();

#[embassy_executor::task]
async fn bob() {
    let mut bob = Router::<SimpleExample>::new("bob".to_string());
    bob.links.insert(1, Neighbour::new("eve".to_string()));
    let rx = RADIOWAVES.receiver();
    let tx = RADIOWAVES.sender();

    loop {
        // read from the channel - do we have any messages addressed to us?
        let _ = match rx.try_receive() {
            Ok(OutboundPacket { link, dest, packet }) => {
                // If this outbound packet was destined for us!
                if dest == bob.address {
                    match bob.handle_packet(&packet, &link, &dest) {
                        Ok(_) => {}
                        Err(e) => {
                            println!("{} handle error: {:?}", bob.address, e);
                        }
                    }
                }
            }
            Err(_) => {}
        };

        // collect all of our packets, if any
        let packets: Vec<OutboundPacket<SimpleExample>> = bob.outbound_packets.drain(..).collect();

        // bob is trying to send a packet. lets try deliver it.
        for pkt in packets {
            // println!("bob sends: {:?}", &pkt);
            let _ = tx.try_send(pkt).ok();
        }

        // performs route table calculations, and writes routing updates into outbound_packets
        bob.full_update();

        // print the routing table
        println!("=== {} routing table ===", bob.address);
        for (
            neigh,
            Route::<SimpleExample> {
                metric, next_hop, ..
            },
        ) in &bob.routes
        {
            println!(" - {neigh}: metric: {metric}, next_hop: {next_hop}")
        }
        println!("");

        // Wait a bit
        Timer::after_secs(1).await;
    }
}

#[embassy_executor::task]
async fn eve() {
    let mut eve = Router::<SimpleExample>::new("eve".to_string());
    eve.links.insert(1, Neighbour::new("bob".to_string()));
    eve.links.insert(2, Neighbour::new("alice".to_string()));

    let rx = RADIOWAVES.receiver();
    let tx = RADIOWAVES.sender();

    loop {
        // read from the channel - do we have any messages addressed to us?
        let inpkt: OutboundPacket<SimpleExample> = rx.receive().await;

        if inpkt.dest == eve.address {
            match eve.handle_packet(&inpkt.packet, &inpkt.link, &inpkt.dest) {
                Ok(_) => {}
                Err(e) => {
                    println!("{} handle error: {:?}", eve.address, e);
                }
            }
        }

        let packets: Vec<OutboundPacket<SimpleExample>> = eve.outbound_packets.drain(..).collect();
        for pkt in packets {
            // println!("eve sends: {:?}", &pkt);
            let _ = tx.try_send(pkt).ok();
        }

        // performs route table calculations, and writes routing updates into outbound_packets
        eve.full_update();

        // print the routing table
        println!("=== {} routing table ===", eve.address);
        for (
            neigh,
            Route::<SimpleExample> {
                metric, next_hop, ..
            },
        ) in &eve.routes
        {
            println!(" - {neigh}: metric: {metric}, next_hop: {next_hop}")
        }
        println!("");

        Timer::after_secs(1).await;
    }
}

#[embassy_executor::task]
async fn alice() {
    let mut alice = Router::<SimpleExample>::new("alice".to_string());
    alice.links.insert(2, Neighbour::new("eve".to_string()));

    let rx = RADIOWAVES.receiver();
    let tx = RADIOWAVES.sender();

    loop {
        // read from the channel - do we have any messages addressed to us?
        let inpkt: OutboundPacket<SimpleExample> = rx.receive().await;

        if inpkt.dest == alice.address {
            match alice.handle_packet(&inpkt.packet, &inpkt.link, &inpkt.dest) {
                Ok(_) => {}
                Err(e) => {
                    println!("{} handle error: {:?}", alice.address, e);
                }
            }
        }

        let packets: Vec<OutboundPacket<SimpleExample>> =
            alice.outbound_packets.drain(..).collect();
        for pkt in packets {
            // println!("eve sends: {:?}", &pkt);
            let _ = tx.try_send(pkt).ok();
        }

        // performs route table calculations, and writes routing updates into outbound_packets
        alice.full_update();

        // print the routing table
        println!("=== {} routing table ===", alice.address);
        for (
            neigh,
            Route::<SimpleExample> {
                metric, next_hop, ..
            },
        ) in &alice.routes
        {
            println!(" - {neigh}: metric: {metric}, next_hop: {next_hop}")
        }
        println!("");

        Timer::after_secs(1).await;
    }
}

static EXEC: StaticCell<Executor> = StaticCell::new();

fn main() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .filter_module("async_io", log::LevelFilter::Info)
        .format_timestamp_nanos()
        .init();

    let executor = EXEC.init(Executor::new());
    executor.run(|spawner| {
        spawner.spawn(bob()).unwrap();
        spawner.spawn(eve()).unwrap();
        spawner.spawn(alice()).unwrap();
    });
}
