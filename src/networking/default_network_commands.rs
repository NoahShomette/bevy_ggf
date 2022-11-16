use bevy::prelude::*;

#[allow(unused)]
pub fn client_register_default_network_messages(app: &mut App) {
    use bevy_spicy_networking::AppNetworkClientMessage;

    // The client registers messages that arrives from the server, so that
    // it is prepared to handle them. Otherwise, an error occurs.
    //app.listen_for_client_message::<NewChatMessage>();
}

#[allow(unused)]
pub fn server_register_default_network_messages(app: &mut App) {
    use bevy_spicy_networking::AppNetworkServerMessage;

    // The server registers messages that arrives from a client, so that
    // it is prepared to handle them. Otherwise, an error occurs.
    //app.listen_for_server_message::<UserChatMessage>();
}