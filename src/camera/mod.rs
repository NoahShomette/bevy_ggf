//! # Bevy_GGF/camera
//! Simply add the [`GGFCameraPlugin`] to your app to get a working 2d camera with click to drag
//! movement and support for left click, right click, and left click hold.
//!
//! Alternatively if you don't want to use the built in plugin you are free to create your own
//! camera however you like. However if you want to use the built in selection manager your plugin
//! will have to emit the [`ClickEvent`] events and work through that system.
//!

use bevy::render::camera::RenderTarget;
use bevy::{math::Vec3, prelude::*, render::camera::Camera};
use leafwing_input_manager::prelude::*;
use leafwing_input_manager::user_input::InputKind::Mouse;

/// A plugin containing the systems and resources for the Bevy_GGF camera system to function
pub struct BggfCameraPlugin;

impl Plugin for BggfCameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<CameraAndCursorInformation>()
            .init_resource::<CursorWorldPos>()
            .add_event::<ClickEvent>()
            .add_plugin(InputManagerPlugin::<CameraMovementAction>::default())
            .add_startup_system(startup)
            .add_system(camera_logic)
            .add_system(click_handler.before(camera_logic))
            .add_system(handle_camera_movement.before(camera_logic))
            .add_system(update_cursor_world_pos);
    }
}

/* Temp storage. Want to figure out a way to make the camera be customizable - so you can set it to
    pixel perfect or whatever else is wanted
pub struct GGFCameraPlugins;

impl PluginGroup for GGFCameraPlugins{
    fn build(self) -> PluginGroupBuilder {
        PluginGroupBuilder::na
    }

    fn set<T: Plugin>(self, plugin: T) -> PluginGroupBuilder {

    }
}

 */

/// An enum used to represent what kind of Camera to spawn
pub enum CameraType {
    Pixel2dCamera,
    Standard,
}

/// Camera Bundle that incorporates the base Bevy Camera2D as well as any additional components needed
#[derive(Bundle)]
struct GGFCamera2dBundle {
    camera_2d_bundle: Camera2dBundle,
}

/// How long the left mouse button needs to be held before its registered as a left click hold event
const CLICK_HOLD_TIME: f32 = 0.5;
/// The distance that the cursor must be dragged after clicking in order to register it as attempting
/// to move the camera
const CLICK_DRAG_MIN_DISTANCE: f32 = 5.0;

/// An enum representing the cameras actions used by Leafwing Input Manager
#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug)]
enum CameraMovementAction {
    Click,
    Zoom,
    RightClick,
}

/// An enum representing the current camera state
#[derive(PartialEq, Eq, Clone, Copy, Hash, Debug)]
enum CameraState {
    None,
    LeftClickInitial,
    Dragging,
    LeftClick,
    LeftClickHold,
    RightClick,
}

#[derive(PartialEq, Clone, Copy, Debug, Default, Resource)]
pub struct CursorWorldPos {
    pub cursor_world_pos: Vec2,
}

/// An event sent when the left clicking, right clicking, or holding left click
pub enum ClickEvent {
    Click { world_pos: Vec2 },
    Hold { world_pos: Vec2 },
    RightClick { world_pos: Vec2 },
}

/// Holds information needed by the camera logic and handler functions
#[derive(Resource)]
struct CameraAndCursorInformation {
    last_frame_cursor_position: Vec2,
    last_click_cursor_position: Vec2,
    camera_state: CameraState,
}

impl Default for CameraAndCursorInformation {
    fn default() -> Self {
        CameraAndCursorInformation {
            last_frame_cursor_position: Vec2 { x: 0.0, y: 0.0 },
            last_click_cursor_position: Vec2 { x: 0.0, y: 0.0 },
            camera_state: CameraState::None,
        }
    }
}

fn startup(mut commands: Commands) {
    commands
        .spawn(GGFCamera2dBundle {
            camera_2d_bundle: Camera2dBundle::default(),
        })
        .insert(InputManagerBundle {
            action_state: ActionState::default(),
            input_map: InputMap::default()
                .insert(SingleAxis::mouse_wheel_y(), CameraMovementAction::Zoom)
                .insert(Mouse(MouseButton::Left), CameraMovementAction::Click)
                .insert(Mouse(MouseButton::Right), CameraMovementAction::RightClick)
                .build(),
        });
}

/// A simple logic system for setting the camera state to the right state. Handles the logic and then
/// separate functions run that logic
/// Handles the zoom for now
fn camera_logic(
    mut query: Query<(
        &mut OrthographicProjection,
        &ActionState<CameraMovementAction>,
        &Camera,
    )>,
    mut camera_cursor_information: ResMut<CameraAndCursorInformation>,
    windows: Res<Windows>,
) {
    let (mut ortho, action_state, camera) = query.single_mut();
    const CAMERA_ZOOM_RATE: f32 = 0.05;

    // get current window - used to get the mouse cursors position for click events and drag movement
    let wnd = if let RenderTarget::Window(id) = camera.target {
        windows.get(id).unwrap()
    } else {
        windows.get_primary().unwrap()
    };
    //if the cursor is inside the current window then we want to handle any clicks that it might do
    if let Some(current_cursor_position) = wnd.cursor_position() {
        // Saves the cursor position when the mouse is clicked
        if action_state.just_pressed(CameraMovementAction::Click) {
            camera_cursor_information.last_click_cursor_position = wnd.cursor_position().unwrap();
            camera_cursor_information.camera_state = CameraState::LeftClickInitial;
        }

        // Calculates the dif for the position of the mouse when first clicked and the current position
        // used to calculate if the player has dragged the mouse away from the starting position enough
        // to activate the drag camera action
        let x_dif =
            camera_cursor_information.last_click_cursor_position.x - current_cursor_position.x;
        let y_dif =
            camera_cursor_information.last_click_cursor_position.y - current_cursor_position.y;

        let mut did_left_click_hold = false;
        let mut is_moving_camera = false;

        // If we are still in the left click initial phase before we've decided what action we are
        // taking then we want to check our main conditions
        if camera_cursor_information.camera_state == CameraState::LeftClickInitial {
            is_moving_camera = x_dif > CLICK_DRAG_MIN_DISTANCE
                || y_dif > CLICK_DRAG_MIN_DISTANCE
                || x_dif < -CLICK_DRAG_MIN_DISTANCE
                || y_dif < -CLICK_DRAG_MIN_DISTANCE;

            let left_click_hold_duration = action_state
                .current_duration(CameraMovementAction::Click)
                .as_secs_f32();

            did_left_click_hold = left_click_hold_duration > CLICK_HOLD_TIME;
        }

        // Handles the logic if we just do a long hold of the left click.
        if action_state.pressed(CameraMovementAction::Click) && did_left_click_hold {
            camera_cursor_information.camera_state = CameraState::LeftClickHold;
        }
        // Handles camera movement logic
        else if action_state.pressed(CameraMovementAction::Click) && is_moving_camera {
            camera_cursor_information.camera_state = CameraState::Dragging;
        } else if camera_cursor_information.camera_state == CameraState::LeftClickInitial
            && action_state.just_released(CameraMovementAction::Click)
        {
            camera_cursor_information.camera_state = CameraState::LeftClick;
        }

        if action_state.just_released(CameraMovementAction::RightClick)
            && !is_moving_camera
            && camera_cursor_information.camera_state != CameraState::Dragging
        {
            camera_cursor_information.camera_state = CameraState::RightClick;
        }

        if camera_cursor_information.camera_state != CameraState::LeftClick
            && action_state.just_released(CameraMovementAction::Click)
        {
            camera_cursor_information.camera_state = CameraState::None;
        }

        camera_cursor_information.last_frame_cursor_position = current_cursor_position;
    }

    let zoom_delta = action_state.value(CameraMovementAction::Zoom);
    ortho.scale *= 1. - zoom_delta * CAMERA_ZOOM_RATE;
}

/// Handles sending click events when we are in the right click state as determined by the [`camera_logic`]
/// function
fn click_handler(
    mut query: Query<(&GlobalTransform, &Camera)>,
    mut camera_cursor_information: ResMut<CameraAndCursorInformation>,
    windows: Res<Windows>,
    mut click_event_writer: EventWriter<ClickEvent>,
) {
    let (global_transform, camera) = query.single_mut();

    // get current window - used to get the mouse cursors position for click events and drag movement
    let wnd = if let RenderTarget::Window(id) = camera.target {
        windows.get(id).unwrap()
    } else {
        windows.get_primary().unwrap()
    };
    //if the cursor is inside the current window then we want to handle any clicks that it might do
    if let Some(current_cursor_position) = wnd.cursor_position() {
        match camera_cursor_information.camera_state {
            CameraState::LeftClick => {
                info!("Left Click");
                let ray = camera
                    .viewport_to_world(global_transform, current_cursor_position)
                    .unwrap();
                let new_position = ray.origin.truncate();

                click_event_writer.send(ClickEvent::Click {
                    world_pos: new_position,
                });
                camera_cursor_information.camera_state = CameraState::None;
            }
            CameraState::LeftClickHold => {
                info!("Left Click Hold");

                let ray = camera
                    .viewport_to_world(global_transform, current_cursor_position)
                    .unwrap();
                let new_position = ray.origin.truncate();

                click_event_writer.send(ClickEvent::Hold {
                    world_pos: new_position,
                });
                camera_cursor_information.camera_state = CameraState::None;
            }
            CameraState::RightClick => {
                info!("Right Click");

                let ray = camera
                    .viewport_to_world(global_transform, current_cursor_position)
                    .unwrap();
                let new_position = ray.origin.truncate();

                click_event_writer.send(ClickEvent::RightClick {
                    world_pos: new_position,
                });
                camera_cursor_information.camera_state = CameraState::None;
            }
            _ => {}
        }
    }
}

/// Handles camera movement when the camera state is in the draggin state
fn handle_camera_movement(
    mut query: Query<(&mut Transform, &GlobalTransform, &Camera)>,
    camera_cursor_information: ResMut<CameraAndCursorInformation>,
    windows: Res<Windows>,
) {
    let (mut transform, global_transform, camera) = query.single_mut();

    // get current window - used to get the mouse cursors position for click events and drag movement
    let wnd = if let RenderTarget::Window(id) = camera.target {
        windows.get(id).unwrap()
    } else {
        windows.get_primary().unwrap()
    };

    //if the cursor is inside the current window then we want to handle any clicks that it might do
    if let Some(current_cursor_position) = wnd.cursor_position() {
        let window_size = Vec2::new(wnd.width(), wnd.height());
        if camera_cursor_information.camera_state == CameraState::Dragging {
            info!("Dragging");

            //info!("ccp: {}", current_cursor_position);
            //info!("lcp: {}",camera_cursor_information.last_frame_cursor_position);
            let x_dif =
                camera_cursor_information.last_frame_cursor_position.x - current_cursor_position.x;
            let y_dif =
                camera_cursor_information.last_frame_cursor_position.y - current_cursor_position.y;

            let position_to_get_world_point = Vec2 {
                x: window_size.x / 2.0 + x_dif,
                y: window_size.y / 2.0 + y_dif,
            };

            let ray = camera
                .viewport_to_world(global_transform, position_to_get_world_point)
                .unwrap();
            let new_position = ray.origin.truncate();

            let new_position = Vec3 {
                x: new_position.x,
                y: new_position.y,
                z: transform.translation.z,
            };

            transform.translation = new_position;
        }
    }
}

fn update_cursor_world_pos(
    mut query: Query<(&GlobalTransform, &Camera)>,
    mut cursor_world_pos: ResMut<CursorWorldPos>,
    windows: Res<Windows>,
) {
    let (global_transform, camera) = query.single_mut();

    // get current window - used to get the mouse cursors position for click events and drag movement
    let wnd = if let RenderTarget::Window(id) = camera.target {
        windows.get(id).unwrap()
    } else {
        windows.get_primary().unwrap()
    };

    //if the cursor is inside the current window then we want to update the cursor position
    if let Some(current_cursor_position) = wnd.cursor_position() {
        let ray = camera
            .viewport_to_world(global_transform, current_cursor_position)
            .unwrap();
        cursor_world_pos.cursor_world_pos = ray.origin.truncate();
    }
}
