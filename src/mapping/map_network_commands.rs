use std::fs::File;
use bevy::prelude::{App, DynamicScene, Handle, Scene};
use bevy_spicy_networking::{ClientMessage, NetworkMessage, ServerMessage};
use serde::{Deserialize, Serialize};

#[allow(unused)]
pub fn client_register_default_mapping_messages(app: &mut App) {
    use bevy_spicy_networking::AppNetworkClientMessage;

    // The client registers messages that arrives from the server, so that
    // it is prepared to handle them. Otherwise, an error occurs.
    app.listen_for_client_message::<SendClientFullMap>();
}

#[allow(unused)]
pub fn server_register_default_mapping_messages(app: &mut App) {
    use bevy_spicy_networking::AppNetworkServerMessage;

    // The server registers messages that arrives from a client, so that
    // it is prepared to handle them. Otherwise, an error occurs.
    app.listen_for_server_message::<ClientRequestFullMap>();
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ClientRequestFullMap;

#[typetag::serde]
impl NetworkMessage for ClientRequestFullMap{
}

impl ServerMessage for ClientRequestFullMap {
    const NAME: &'static str = "bevy_ggf:ClientRequestFullMap";
}


#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct SendClientFullMap{
    //map_scene: File,
}
#[typetag::serde]
impl NetworkMessage for SendClientFullMap{
}

impl ClientMessage for SendClientFullMap {
    const NAME: &'static str = "bevy_ggf:SendClientFullMap";
}