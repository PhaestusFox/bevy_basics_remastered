use bevy::{
    input::{common_conditions::input_just_released, mouse::AccumulatedMouseMotion},
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow, WindowFocused},
};

const SPEED: f32 = 50.;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_systems(Startup, (spawn_map, spawn_camera));
    app.add_systems(
        Update,
        (
            // make the player look around
            player_look,
            // look for winit focus/unfocused events
            focus_events,
            // toggle focus when you press escape - shows of run conditions
            toggle_grab.run_if(input_just_released(KeyCode::Escape)),
            // move player in the direction they are looking
            player_move.after(player_look),
        ),
    );
    app.add_observer(apply_grab);
    app.run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((Camera3d::default(), Player));
}

fn spawn_map(
    mut commands: Commands,
    mut mesh_assets: ResMut<Assets<Mesh>>,
    mut material_assets: ResMut<Assets<StandardMaterial>>,
) {
    commands.spawn(DirectionalLight::default());
    let ball_mesh = mesh_assets.add(Sphere::new(1.));
    for h in 0..16 {
        let color = Color::hsl((h as f32 / 16.) * 360., 1., 0.5);
        let ball_material = material_assets.add(StandardMaterial {
            base_color: color,
            ..Default::default()
        });
        commands.spawn((
            Transform::from_translation(Vec3::new((-8. + h as f32) * 2., 0., -50.0)),
            Mesh3d(ball_mesh.clone()),
            MeshMaterial3d(ball_material),
        ));
    }
}

#[derive(Component)]
struct Player;

fn player_look(
    // should only be on player you control so use single so save unwrapping
    // if no player is found this system will not run
    // if more then one player is found the system will also not run
    mut player: Single<&mut Transform, With<Player>>,
    //can use raw events
    // mut mouse_motion: EventReader<bevy::input::mouse::MouseMotion>,
    mouse_movement: Res<AccumulatedMouseMotion>,
    // use delta time so mouse is consistent even when game is slow or at non 60 fps
    time: Res<Time>,
    // use window to check if we should let the player look or not
    window: Single<&Window, With<PrimaryWindow>>,
) {
    // if using MouseMotion events need to accumulate them
    // let delta = mouse_motion.read().map(|e| e.delta).sum();

    // if window is not focused don't let player look
    if !window.focused {
        return;
    }

    // change to use 100. divided by min width and hight, this will make the game feel the same even on different resolutions
    let sensitivity = 100. / window.width().min(window.height());

    //get angles as euler angles because they are more natural then Quats, don't need role
    let (mut yaw, mut pitch, _) = player.rotation.to_euler(EulerRot::YXZ);
    // subtract y movement for pitch - up/down
    pitch -= mouse_movement.delta.y * time.delta_secs() * sensitivity;

    // subtract x movement for yaw - left/right
    yaw -= mouse_movement.delta.x * time.delta_secs() * sensitivity;

    // stops you looking past straight up, it will flickering as the value becomes negative
    pitch = pitch.clamp(-1.57, 1.57);

    // recalculate the Quat from the yaw and pitch, yaw first or we end up with unintended role
    player.rotation = Quat::from_euler(EulerRot::YXZ, yaw, pitch, 0.);
}

#[derive(Event, Deref)]
struct GrabEvent(bool);

fn apply_grab(
    // tells bevy what event to watch for with this observer
    grab: Trigger<GrabEvent>,
    mut window: Single<&mut Window, With<PrimaryWindow>>,
) {
    if **grab {
        window.cursor_options.visible = false;
        window.cursor_options.grab_mode = CursorGrabMode::Locked
    } else {
        window.cursor_options.visible = true;
        window.cursor_options.grab_mode = CursorGrabMode::None;
    }
}

fn focus_events(mut events: EventReader<WindowFocused>, mut commands: Commands) {
    if let Some(event) = events.read().last() {
        commands.trigger(GrabEvent(event.focused));
    }
}

fn toggle_grab(mut window: Single<&mut Window, With<PrimaryWindow>>, mut commands: Commands) {
    window.focused = !window.focused;
    commands.trigger(GrabEvent(window.focused));
}

fn player_move(
    // need access to player
    mut player: Single<&mut Transform, With<Player>>,
    // need access to keyboard inputs
    inputs: Res<ButtonInput<KeyCode>>,
    // need delta time to update position consistently even during lag or non 60 fps
    time: Res<Time>,
) {
    let mut delta = Vec3::ZERO;
    if inputs.pressed(KeyCode::KeyA) {
        delta.x -= 1.;
    }
    if inputs.pressed(KeyCode::KeyD) {
        delta.x += 1.;
    }
    if inputs.pressed(KeyCode::KeyW) {
        delta.z += 1.;
    }
    if inputs.pressed(KeyCode::KeyS) {
        delta.z -= 1.;
    }

    let forward = player.forward().as_vec3() * delta.z;
    let left = player.right().as_vec3() * delta.x;
    let mut to_move = forward + left;
    to_move.y = 0.;
    player.translation += to_move.normalize_or_zero() * time.delta_secs() * SPEED;
}