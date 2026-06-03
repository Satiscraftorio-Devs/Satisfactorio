use network::messages::BroadcastMessage;
use tokio::sync::broadcast;

const BROADCAST_CAPACITY: usize = 512;

pub fn channel() -> (broadcast::Sender<BroadcastMessage>, broadcast::Receiver<BroadcastMessage>) {
    broadcast::channel(BROADCAST_CAPACITY)
}
