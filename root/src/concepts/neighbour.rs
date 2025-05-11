use hashbrown::HashMap;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::concepts::route::ExternalRoute;
use crate::framework::RoutingSystem;

#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize), serde(bound = ""))]
pub struct Neighbour<T: RoutingSystem + ?Sized> {
    /// the routing network address
    pub addr: T::NodeAddress,
    pub routes: HashMap<T::NodeAddress, ExternalRoute<T>>,
    /// Direct Link-metric to this neighbour, 0xFFFF for Infinity. Lower is better.
    /// INF if the link is down
    pub metric: u16,
}

impl<T: RoutingSystem + ?Sized> Neighbour<T> {
    pub fn new(addr: T::NodeAddress) -> Neighbour<T> {
        Self {
            addr,
            routes: Default::default(),
            metric: 1,
        }
    }
}
