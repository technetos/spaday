use std::path::PathBuf;
use std::sync::mpsc::{channel, Sender};

use clap::Parser;
use pnet::datalink::{self, Channel};
use pnet::packet::ethernet::{EtherTypes, EthernetPacket};
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::udp::UdpPacket;
use pnet::packet::Packet;
use spaday::KNOCKKNOCK;

#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    #[arg(short)]
    interface: String,

    #[arg(short)]
    pub_key_file: PathBuf,
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();

    let (tx, rx) = channel::<Vec<u8>>();

    let decoder_thread = std::thread::spawn(move || loop {
        match rx.recv() {
            Ok(bytes) => {
                if let Err(e) = decode_payload(&bytes) {
                    tracing::error!("An error occurred when decrypting the udp payload: {e}");
                }
            }
            Err(e) => {
                tracing::error!(
                    "An error occurred when reading from the decoder thread channel: {e}"
                );
            }
        }
    });

    let interfaces = pnet::datalink::interfaces();
    let interface = interfaces
        .into_iter()
        .filter(|interface| interface.name == args.interface)
        .next()
        .unwrap_or_else(|| panic!("Interface not found"));

    if let Channel::Ethernet(_, mut rx) = datalink::channel(&interface, Default::default())? {
        loop {
            match rx.next() {
                Ok(packet) => {
                    if let Some(packet) = EthernetPacket::new(packet) {
                        handle_packet(&packet, &tx)?;
                    }
                }
                Err(e) => {
                    tracing::error!("An error occurred when receiving a packet: {e}");
                }
            }
        }
    }

    decoder_thread
        .join()
        .expect("Failed to join decoder thread");

    Ok(())
}

fn handle_packet(ethernet: &EthernetPacket, tx: &Sender<Vec<u8>>) -> anyhow::Result<()> {
    if let EtherTypes::Ipv4 = ethernet.get_ethertype() {
        if let Some(header) = Ipv4Packet::new(ethernet.payload()) {
            if let IpNextHeaderProtocols::Udp = header.get_next_level_protocol() {
                if let Some(udp) = UdpPacket::new(header.payload()) {
                    let udp_payload = udp.payload();
                    if udp_payload.starts_with(KNOCKKNOCK) {
                        let bytes = udp_payload[KNOCKKNOCK.len()..].to_vec();
                        if let Err(e) = tx.send(bytes) {
                            tracing::error!(
                                "An error occurred when sending to the decoder thread: {e}"
                            );
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

fn decode_payload(bytes: &[u8]) -> anyhow::Result<()> {
    dbg!(bytes);
    Ok(())
}
