use bevy::{input::mouse::AccumulatedMouseMotion, prelude::*};

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_systems(Startup, (spawn_map, spawn_camera));
    app.add_systems(Update, player_look);
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
) {
    // if using MouseMotion events need to accumulate them
    // let delta = mouse_motion.read().map(|e| e.delta).sum();

    // just set to 0.1 for now
    let sensitivity = 0.1;

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
