use bevy::prelude::*;

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins);
    app.add_systems(Startup, (spawn_map, spawn_camera));
    app.run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn(Camera3d::default());
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
