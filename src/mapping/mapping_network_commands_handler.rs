use bevy::prelude::{Commands, Component, EventReader, info, Res};
use bevy_spicy_networking::{ConnectionId, NetworkServer, ServerNetworkEvent};

#[derive(Component)]
struct Player(ConnectionId);

fn handle_connection_events(
    mut commands: Commands,
    net: Res<NetworkServer>,
    mut network_events: EventReader<ServerNetworkEvent>,
) {
    for event in network_events.iter() {
        if let ServerNetworkEvent::Connected(conn_id) = event {
            commands.spawn((Player(conn_id.clone()),));

            // Broadcasting sends the message to all connected players! (Including the just connected one in this case)
            net.broadcast(shared::NewChatMessage {
                name: String::from("SERVER"),
                message: format!("New user connected; {}", conn_id),
            });
            info!("New player connected: {}", conn_id);
        }
    }
}