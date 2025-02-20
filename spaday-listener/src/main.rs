use clap::Parser;
use pnet::datalink::{self, Channel};
use pnet::packet::ethernet::{EtherTypes, EthernetPacket};
use pnet::packet::ip::IpNextHeaderProtocols;
use pnet::packet::ipv4::Ipv4Packet;
use pnet::packet::udp::UdpPacket;
use pnet::packet::Packet;

#[derive(Parser, Debug)]
#[command(version)]
struct Args {
    #[arg(short)]
    interface: String,
}

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let args = Args::parse();

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
                        handle_packet(&packet)?;
                    }
                }
                Err(e) => {
                    tracing::error!("An error occurred when receiving a packet: {e}");
                }
            }
        }
    }
    Ok(())
}

fn handle_packet(ethernet: &EthernetPacket) -> anyhow::Result<()> {
    if let EtherTypes::Ipv4 = ethernet.get_ethertype() {
        if let Some(header) = Ipv4Packet::new(ethernet.payload()) {
            if let IpNextHeaderProtocols::Udp = header.get_next_level_protocol() {
                if let Some(udp) = UdpPacket::new(header.payload()) {
                    let udp_payload = udp.payload();
                    dbg!(udp_payload);
                }
            }
        }
    }

    Ok(())
}
