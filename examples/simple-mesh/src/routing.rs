use root::framework::RoutingSystem;
use root::router::NoMACSystem;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct IPV4System {}
impl RoutingSystem for IPV4System {
    type NodeAddress = String;
    type Link = Uuid;
    type MACSystem = NoMACSystem;
}
