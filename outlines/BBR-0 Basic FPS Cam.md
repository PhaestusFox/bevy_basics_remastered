Start in Finish Game
Describe Purpose of Remastered Series
Talk About using different tools from the bevy toolbox to show what's there not because they are the best
Talk About watching zero to pong if you want to go from a fresh late this assumes and understanding of setting up a rust project
can follow along with GitHub repo, example ball_fps
each time the game is run will be a commit, the commit name will match chapter tital

1. Setup Cargo.toml
 - Add bevy 0.16
 - Add Rand "\*"
 - Add profile opt-level
2. Setup Basic App
 - use bevy prelude
 - make new mut app in main
 - add `DefaultPlugins` to app
 - run app
> Run game to see default window
3. Make Startup systems
 - spawn_camera
  - takes in commands parameter
  - spawn entity with camera3d::default()
 - spawn_map
  - has Parameters
	  Commands
	  Res\<Assets\<Mesh>>
	  Res\<Assets\<StandardMaterial>>
  - spawn light
  - add sphere mesh to assets
  - spawn 16 balls
  - make color for each ball
  - spawn in a line 50 units in front of player
> Run game to see mouse world
4. Add Player Look
 - create player Component
 - add Player component to Camera
  - make player look system
  - has Parameters
	Single<&mut Transform, With\<Player>>
	Res\<AccumulatedMouseMotion> - Can use mouse events
	time: Res\<Time>
 - create sensitivity for later - 0.1
 - extract yaw and pitch from transform
 - apply pitch - mouse y * delta time * sensitivity
 - apply yaw - mouse x * delta time * sensitivity
 - clamp pitch between -1.57 and 1.57
 - set rotation: do yaw then pitch to prevent unintentional role
 - add system to app
> Run game to see mouse movement
5. Make mouse lock when focused and only move if focused
 - add Single<&Window, With\<PrimaryWindow>> to player look system
 - check if window is focused in garde cluse
 - update sensitivity to 100 / min of window width and window height so sensitivity is that same regardless of window size. because why not.
 - Make Grab event just bool of if focused or not
 - Derive deref for convivence
 - Create apply grab observer system
  - has Parameters
	 Trigger\<GrabEvent>
	 Single<&mut Window, With\<PrimaryWindow>>
 - if focused set lock and hide, if not focused set none and visible
 - add observer to app
 - create focus_events system
  - has Parameters
	 EventReader\<WindowFocused>
	 Commands
 - Iter all focus events triggering GrabEvent
- create toggle_grab system
 -  has Parameters
	  Single<&mut Window, With\<PrimaryWindow>>,
	  Commands,
 - when run sets focused to !focused
 - sends grab event with new focucsed
 - add all systems to game
 - give toggle focus a run condition
> run game to see cursor grab
6. Make player move
 - create const for player move speed
 - create player move system
  - has Parameters
	Single<&mut Transform, With\<Player>>,
	inputs: Res<ButtonInput\<KeyCode>>,
	time: Res\<Time>,
 - get delta from keys being pressed
 - get players forward vector
 - get players left vector
 - multiply vectors by delta for that direction
 - set y of movement to zero so player doesn't fly
 - normalize movement so player moves at consistent speed even when looking up
 - add movement to player translation
- add movement system to game - after player looks to get more consistent feel to movement
> run game to see player move
7. add ball spawning
 - create Ball Spawn Event
 - position to spawn at
 - create Ball spawn system
  - has Parameters
	EventReader\<BallEvent>
	Commands,
	Res\<Assets\<Mesh>>
	Res\<Assets\<StandardMaterial>>
  - read all spawn events
  - spawn ball with translation of spawn location
  - add sphere mesh
  - add material
- create system to send spawn events
 - has Parameters
	Res<ButtonInput\<MouseButton>>,
	Single<&Transform, With\<Player>>,
	EventWriter\<BallSpawn>
 - have garde clause if cursor is visible so ball dose not spawn when clicking back into window 
 - have garde clause if player has left clicked
 - if player has pressed left, spawn ball at there current translation
- add spawn ball system
- add BallSpawn event
- add shoot system, with before ordering rules
> run game to spawn balls
8. create resource to hold premade ball data
 -  make ball data struct
	mesh handle
	materials vec
	 rng mutex - optional for what I'm doing
- impl BallData
	mesh() returns cloned mesh
	material() returns random cloned material
- impl FromWorld on BallDatas
- init balldata in app
- update spawning world to use ball data
- update spawning balls to use ball data
> run game to show nothing changed
9. add velocity to balls
 -  create velocity Component
 - add system that applies velocity each frame
  -  has Parameters
	   Query<(&mut Transform, &Velocity)>
	   Res\<Time>
- add starting velocity to ball spawn event
- update shoot to apply a velocity in the direction the player is looking to the ball
- update ball spawn to add velocity to ball when spawned
> run game to show balls flying around
10. add gravity to balls
- create gravity const
- create gravity system
  -  has Parameters
	   Query<(&mut Velocity)>
 - apply gravity to anything with a velocity every 1/30 of a second
 - add gravity to app in fixed update, runs before applying velocity
 > run game to show gravity
11. add bouncing off ground
 - create bounce system
   -  has Parameters
	   Query<(&Transform, &mut Velocity)>
 - flip velocity if transform is below ground
 - checks if velocity is negative to prevent stuff getting stuck in the ground
 - add system to game, runs after applying velocity if fixed update
 > run game to show bounce
12. add shoot power
 - add power to ball spawn - change velocity to direction
 - add local Option\<f32> to shoot ball system
 - add time to ball system
 - check if ball is already charging
 - if left mouse is released shoot ball with current power
 - if left is still held incress power
 - if left is not held clear power
 - set power to 1. when player pressed left
 - modify spawning ball to use power
> run game to see different powers
13. Add power bar
 - create power resource - insert power
 - create PowerBar component
 - modify shoot to use new power resource
 - create bar color consts
 - create power UI inside setup world
 - ~ intalisens does not work inside children macro
 - have to use VMax 0.125 to get eq margin since there is no way to get percent
 - create update_bar system
 - add update bar to app
 > run game to show finished example
 