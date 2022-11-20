use bevy::input::mouse::MouseMotion;
use bevy::math::vec2;
use bevy::render::camera::RenderTarget;
use bevy::{input::Input, math::Vec3, prelude::*, render::camera::Camera};
use leafwing_input_manager::axislike::MouseMotionAxisType;
use leafwing_input_manager::prelude::*;
use leafwing_input_manager::user_input::InputKind::Mouse;

pub struct GGFCameraPlugin;

impl Plugin for GGFCameraPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<LastCursorPosition>()
            .add_plugin(InputManagerPlugin::<CameraMovementAction>::default())
            .add_startup_system(startup)
            .add_system(movement);
    }
}

#[derive(Actionlike, PartialEq, Eq, Clone, Copy, Hash, Debug)]
pub enum CameraMovementAction {
    Click,
    Pan,
    Zoom,
}

#[derive(Bundle)]
pub struct GGFCameraBundle {}

#[derive(Resource)]
pub struct LastCursorPosition(Vec2);

impl Default for LastCursorPosition {
    fn default() -> Self {
        LastCursorPosition(Vec2 { x: 0.0, y: 0.0 })
    }
}

fn startup(mut commands: Commands) {
    commands
        .spawn(Camera2dBundle::default())
        .insert(InputManagerBundle {
            action_state: ActionState::default(),
            input_map: InputMap::default()
                .insert(DualAxis::mouse_motion(), CameraMovementAction::Pan)
                .insert(SingleAxis::mouse_wheel_y(), CameraMovementAction::Zoom)
                .insert(Mouse(MouseButton::Left), CameraMovementAction::Click)
                .build(),
        });
}

// A simple camera system for moving and zooming the camera.
pub fn movement(
    time: Res<Time>,
    keyboard_input: Res<Input<KeyCode>>,
    mut cursor_event_reader: EventReader<CursorMoved>,
    mut query: Query<(
        &mut Transform,
        &GlobalTransform,
        &mut OrthographicProjection,
        &ActionState<CameraMovementAction>,
        &mut Camera,
    )>,
    mut last_cursor_position: ResMut<LastCursorPosition>,
    windows: Res<Windows>,
) {
    let (mut transform, global_transform, mut ortho, action_state, mut camera) = query.single_mut();
    const CAMERA_PAN_RATE: f32 = 0.5;
    const CAMERA_ZOOM_RATE: f32 = 0.05;

    let wnd = if let RenderTarget::Window(id) = camera.target {
        windows.get(id).unwrap()
    } else {
        windows.get_primary().unwrap()
    };

    // check if the cursor is inside the window and get its position
    if let Some(current_cursor_position) = wnd.cursor_position() {

        let window_size = Vec2::new(wnd.width() as f32, wnd.height() as f32);


        info!("ccp: {}", current_cursor_position);
        info!("lcp: {}", last_cursor_position.0);

        let x_dif = last_cursor_position.0.x - current_cursor_position.x;
        let y_dif = last_cursor_position.0.y - current_cursor_position.y;
        
        //current_cursor_position.x = current_cursor_position.x + x_dif;
        //current_cursor_position.y = current_cursor_position.y + y_dif;

        let position_to_get_world_point = Vec2{
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
        
        // Because we're moving the camera, not the object, we want to pan in the opposite direction
        // However, UI cordinates are inverted on the y-axis, so we need to flip y a second time
        if action_state.pressed(CameraMovementAction::Click) {
            transform.translation = new_position;
        }
        last_cursor_position.0 = current_cursor_position;
    }

    let zoom_delta = action_state.value(CameraMovementAction::Zoom);
    ortho.scale *= 1. - zoom_delta * CAMERA_ZOOM_RATE;
}
