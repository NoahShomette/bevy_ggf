use crate::camera;
use bevy::prelude::*;
use bevy_spicy_networking::{AppNetworkServerMessage, ClientNetworkEvent, ClientPlugin, NetworkClient, NetworkSettings, ServerPlugin};
use crate::networking::default_network_commands::{client_register_default_network_messages, client_register_network_messages, server_register_default_network_messages, server_register_network_messages};

mod client;
mod default_network_commands;
mod server;
mod shared;

#[derive(Default, Copy, Clone, Debug)]
/// The plugin to add to your bevy [`App`](bevy::prelude::App) when you want
/// to instantiate a client. Combines bevy_ggf and Bevy_Spicy_Networking to provide all needed
/// functionality
pub struct GGFClient;

impl Plugin for GGFClient {
    fn build(&self, mut app: &mut App) {
        app.add_plugin(ClientPlugin).add_system(camera::movement);
        
        client_register_default_network_messages(&mut app);
    }
}

#[derive(Default, Copy, Clone, Debug)]
/// The plugin to add to your bevy [`App`](bevy::prelude::App) when you want
/// to instantiate a server. Combines bevy_ggf and Bevy_Spicy_Networking to provide all needed
/// functionality
pub struct GGFServer;

impl Plugin for GGFServer {
    fn build(&self, mut app: &mut App) {
        app.add_plugin(ServerPlugin);
        server_register_default_network_messages(&mut app);
    }
}