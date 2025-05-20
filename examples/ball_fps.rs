use bevy::{
    input::{common_conditions::input_just_released, mouse::AccumulatedMouseMotion},
    prelude::*,
    window::{CursorGrabMode, PrimaryWindow, WindowFocused},
};

const SPEED: f32 = 50.;
// 9.8 is earth gravity and div 30 because 30 steps per second
const GRAVITY: Vec3 = Vec3::new(0., -9.8, 0.);

const NOT_CHARGING: Color = Color::linear_rgb(0.2, 0.2, 0.2);
const MIN_FILL: f32 = 29.75 / 6.;
const EMPTY_SPACE: f32 = 29.75 - MIN_FILL;

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
            // spawn balls when a ball event happens
            spawn_ball,
            // run before spawning ball to prevent potential frame of lag.
            // run before focus events so when we click back in we don't hide the curser before we check the click
            shoot_ball.before(spawn_ball).before(focus_events),
            update_power_bar,
        ),
    );
    // set a fixed update for physics of 30 fps
    // technically things like acceleration don't word correctly if you use a variable delta
    app.insert_resource(Time::<Fixed>::from_hz(30.));
    app.add_systems(
        FixedUpdate,
        (
            // move every entity with a velocity by that amount
            apply_velocity,
            // run before velocity for consistancey between frames,
            apply_gravity.before(apply_velocity),
            bounce.after(apply_velocity),
        ),
    );
    app.add_event::<BallSpawn>();
    app.init_resource::<BallData>();
    app.add_observer(apply_grab);
    app.insert_resource(Power {
        charging: false,
        current: 0.,
    });
    app.run();
}

fn spawn_camera(mut commands: Commands) {
    commands.spawn((Camera3d::default(), Player));
}

fn spawn_map(mut commands: Commands, ball_data: Res<BallData>) {
    commands.spawn(DirectionalLight::default());
    for h in 0..ball_data.materials.len() {
        commands.spawn((
            Transform::from_translation(Vec3::new(
                (-(ball_data.materials.len() as f32) * 0.5 + h as f32) * 2.,
                0.,
                -50.0,
            )),
            Mesh3d(ball_data.mesh()),
            MeshMaterial3d(ball_data.materials[h].clone()),
        ));
    }
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                width: Val::VMax(30.),
                height: Val::VMax(5.),
                bottom: Val::Px(20.),
                left: Val::Px(20.),
                ..Default::default()
            },
            BackgroundColor(Color::linear_rgb(0.5, 0.5, 0.5)),
            BorderRadius::all(Val::VMax(5.)),
        ))
        .with_child((
            Node {
                position_type: PositionType::Absolute,
                min_width: Val::VMax(MIN_FILL),
                height: Val::Percent(95.),
                margin: UiRect::all(Val::VMax(0.125)),
                ..Default::default()
            },
            BackgroundColor(NOT_CHARGING),
            BorderRadius::all(Val::VMax(5.)),
            PowerBar { min: 1., max: 6. },
        ));
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

#[derive(Event)]
struct BallSpawn {
    position: Vec3,
    velocity: Vec3,
    power: f32,
}

fn spawn_ball(
    mut events: EventReader<BallSpawn>,
    mut commands: Commands,
    ball_data: Res<BallData>,
) {
    for spawn in events.read() {
        commands.spawn((
            Transform::from_translation(spawn.position),
            Mesh3d(ball_data.mesh()),
            MeshMaterial3d(ball_data.material()),
            Velocity(spawn.velocity * spawn.power * 10.),
        ));
    }
}

fn shoot_ball(
    inputs: Res<ButtonInput<MouseButton>>,
    player: Single<&Transform, With<Player>>,
    mut spawner: EventWriter<BallSpawn>,
    window: Single<&Window, With<PrimaryWindow>>,
    mut power: ResMut<Power>,
    time: Res<Time>,
) {
    if window.cursor_options.visible {
        return;
    }

    if power.charging {
        if inputs.just_released(MouseButton::Left) {
            spawner.write(BallSpawn {
                position: player.translation,
                velocity: player.forward().as_vec3(),
                power: power.current,
            });
        }
        if inputs.pressed(MouseButton::Left) {
            power.current += time.delta_secs();
            power.current = power.current.clamp(1., 6.);
        } else {
            power.charging = false;
        }
    }

    if inputs.just_pressed(MouseButton::Left) {
        power.charging = true;
        power.current = 1.;
    }
}

#[derive(Resource)]
struct BallData {
    mesh: Handle<Mesh>,
    materials: Vec<Handle<StandardMaterial>>,
    rng: std::sync::Mutex<rand::rngs::StdRng>,
}

impl BallData {
    fn mesh(&self) -> Handle<Mesh> {
        self.mesh.clone()
    }
    fn material(&self) -> Handle<StandardMaterial> {
        use rand::seq::SliceRandom;
        let mut rng = self.rng.lock().unwrap();
        self.materials.choose(&mut *rng).unwrap().clone()
    }
}

impl FromWorld for BallData {
    fn from_world(world: &mut World) -> Self {
        use rand::SeedableRng;
        let mesh = world.resource_mut::<Assets<Mesh>>().add(Sphere::new(1.));
        let mut materials = Vec::new();
        let mut material_assets = world.resource_mut::<Assets<StandardMaterial>>();
        for i in 0..36 {
            let color = Color::hsl((i * 10) as f32, 1., 0.5);
            materials.push(material_assets.add(StandardMaterial {
                base_color: color,
                ..Default::default()
            }));
        }
        let seed = *b"PhaestusFoxBevyBasicsRemastered0";
        BallData {
            mesh,
            materials,
            rng: std::sync::Mutex::new(rand::rngs::StdRng::from_seed(seed)),
        }
    }
}

#[derive(Component, Deref, DerefMut)]
struct Velocity(Vec3);

fn apply_velocity(mut objects: Query<(&mut Transform, &Velocity)>, time: Res<Time>) {
    for (mut transform, velocity) in &mut objects {
        // use delta time because we set velocity outside of fixed update
        // if we did not used delta time then anything touching velocity would need to make sure it is scaled correctly
        transform.translation += velocity.0 * time.delta_secs();
    }
}

// no need for delta time because it is fixed at 1/30
fn apply_gravity(mut objects: Query<&mut Velocity>, time: Res<Time>) {
    let g = GRAVITY * time.delta_secs();
    for mut velocity in &mut objects {
        **velocity += g;
    }
}

fn bounce(mut balls: Query<(&Transform, &mut Velocity)>) {
    for (transform, mut velocity) in &mut balls {
        if transform.translation.y < 0. && velocity.y < 0. {
            velocity.y *= -1.;
        }
    }
}

#[derive(Resource)]
struct Power {
    charging: bool,
    current: f32,
}

#[derive(Component)]
struct PowerBar {
    min: f32,
    max: f32,
}

fn update_power_bar(
    mut bars: Query<(&mut Node, &PowerBar, &mut BackgroundColor)>,
    power: Res<Power>,
) {
    for (mut bar, config, mut bg) in &mut bars {
        if !power.charging {
            bg.0 = NOT_CHARGING;
            bar.width = Val::VMax(MIN_FILL);
        } else {
            let percent = (power.current - config.min) / (config.max - config.min);
            bg.0 = Color::linear_rgb(1. - percent, percent, 0.);
            bar.width = Val::VMax(MIN_FILL + percent * EMPTY_SPACE);
        }
    }
}
