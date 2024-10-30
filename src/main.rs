use bevy::{prelude::*,
    asset::{AssetEvent, Assets, Handle},
    input::common_conditions::*,
    pbr::{CascadeShadowConfigBuilder, DirectionalLightShadowMap},
};

fn main() {
    App::new()
        .insert_resource(DirectionalLightShadowMap { size: 4096 })
        .insert_resource(OpIndex::new())
        .init_resource::<CurrentMeshColor>()
        .init_resource::<Countdown>()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, (
            animate_light_direction,
            handle_asset_events,
            screen_albedo, 
            update_screen_albedo.run_if(input_just_released(MouseButton::Left)),
        ))
        .run();
}

#[derive(Debug, Resource)]
enum MeshColor { // If changed update VARIANT_COUNT 
    Black,
    White,
    Red,
    Green,
    Blue,
}

impl MeshColor {
    const VARIANT_COUNT: u32 = 4;
}

#[derive(Component)]
struct CameraUi;

#[derive(Component)]
struct ColorChange;

#[derive(Resource)]
struct Countdown {
    timer: Timer,           // Set single timer for countdown
    loop_count: u32,        // Number of loops, currently tied to the varient_count to loop through all dynamically
    current_count: u32,     // Tracks where in the loop you are
    is_active: bool,        // Tracks if the loop is active
}

#[derive(Default, Resource)]
struct CurrentMeshColor;

#[derive(Asset, Component, TypePath)]
struct Interactable; 

#[derive(Component)]
struct Loaded;

#[derive(Clone, Resource)]
struct OpIndex {
    index: u32,
}

impl Countdown {
    pub fn new() -> Self {
        Self {
            timer: Timer::from_seconds(1.0 / 3.0, TimerMode::Once), // Set single timer for countdown
            loop_count: MeshColor::VARIANT_COUNT + 1, // +1 accounts for indexed logic
            current_count: 0,
            is_active: false,  // Initially inactive
        }
    }
}

impl Default for Countdown {
    fn default() -> Self {
        Self::new()
    }
}

impl CurrentMeshColor {
    fn from_index(index: u32) -> Option<MeshColor> {
        match index {
            0 => Some(MeshColor::Black),
            1 => Some(MeshColor::White),
            2 => Some(MeshColor::Red),
            3 => Some(MeshColor::Green),
            4 => Some(MeshColor::Blue),
            _ => None, // Handle invalid index
        }
    }

    fn update_current_mesh_color(
        op: &mut ResMut<OpIndex>,
    ) -> Color {
        if let Some(call) = CurrentMeshColor::from_index(op.index) {
            match call {
                MeshColor::Black => {
                    Color::srgb(0.0, 0.0, 0.0)
                },
                MeshColor::White => {
                    Color::srgb(1.0, 1.0, 1.0)
                },
                MeshColor::Red => {
                    Color::srgb(1.0, 0.0, 0.0)
                },
                MeshColor::Green => {
                    Color::srgb(0.0, 1.0, 0.0)
                },
                MeshColor::Blue => {
                    Color::srgb(0.0, 0.0, 1.0)
                },
            }
        } else {
            Color::srgb(0.0, 0.0, 0.0)
        }
    }

    fn update_gltf_material_color(
        children_query: Query<&Children>,
        color_change_cube_query: Query<(Entity, &Handle<Scene>), (With<ColorChange>, With<Loaded>)>,
        mut materials: ResMut<Assets<StandardMaterial>>,
        material_query: Query<&Handle<StandardMaterial>>,
        op_index: &mut ResMut<OpIndex>,
    ) {
        for (entity, _) in color_change_cube_query.iter() {
            if let Ok(children) = children_query.get(entity) {
                Self::process_entity_children(
                    &mut materials,
                    &material_query,
                    children,
                    &children_query,
                    op_index,         
                );
            }
        }
    }

    fn process_entity_children(
        materials: &mut ResMut<Assets<StandardMaterial>>,
        material_query: &Query<&Handle<StandardMaterial>>,
        children: &Children,
        children_query: &Query<&Children>,
        op_index: &mut ResMut<OpIndex>,
    ) {
        for &child in children.iter() {
            if child.index() == 64 { // This targets the screen component specifically, still learning about glb files and how to extract names s I don't have a more dynamic way of handling it for now.
                if let Ok(material_handle) = material_query.get(child) {
                    if let Some(material) = materials.get_mut(material_handle) {
                        material.base_color = CurrentMeshColor::update_current_mesh_color(op_index);
                    }
                }
            }
            // Recursively check grandchildren
            if let Ok(grandchildren) = children_query.get(child) {
                Self::process_entity_children(
                    materials,
                    material_query,
                    grandchildren,
                    children_query,
                    op_index,
                );
            }
        }
    }
}

impl OpIndex {
    fn new() -> Self {
        let index: u32 = 0;
        OpIndex {
            index,
        }
    }
}

fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    // UI Setup
    let font = asset_server.load("fonts/MatrixtypeDisplay-KVELZ.ttf");

    let text_style = TextStyle {
        font: font.clone(),
        font_size: 42.0,
        ..default()
    };
    let smaller_text_style = TextStyle {
        font: font.clone(),
        font_size: 25.0,
        ..default()
    };
    
    // UI Cam
    commands.spawn((
        Camera2dBundle {
            transform: Transform::from_xyz(0.0, 0.0, 0.0),
            camera: Camera {
                order: 1, // Render on top of the 3D scene
                ..default()
                },
            ..default()
        },
        CameraUi,
    ));

    commands.spawn(NodeBundle {
        style: Style {
            display: Display::Flex,
            align_items: AlignItems::Center,    // Center vertically within the container
            justify_content: JustifyContent::Center, // Center horizontally within the container
            position_type: PositionType::Absolute,
            // Set this node to occupy the entire screen
            width: Val::Percent(100.0),   // Use width instead of size
            height: Val::Percent(100.0),  // Use height instead of size
            ..default()
        },
        ..default()
    })
    .with_children(|parent| {
        parent.spawn(TextBundle {
            text: Text {
                sections: vec![TextSection::new(
                    "Calc-Sim...",
                    text_style.clone(),
                )],
                ..default()
            },
            style: Style {
                position_type: PositionType::Absolute,
                // Manually set the position of the text to the bottom left
                // align_self: AlignSelf::Center, // Center horizontally relative to its own width
                top: Val::Percent(2.0),  // 10 pixels from the top
                // left: Val::Percent(50.0),  // 50% from the left
                ..default()
            },
            ..default()
        });
    })
    .with_children(|parent| {
        parent.spawn(TextBundle {
            text: Text {
                sections: vec![TextSection::new(
                    "Left Click to change screen colors...",
                    smaller_text_style.clone(),
                )],
                ..default()
            },
            style: Style {
                position_type: PositionType::Absolute,
                // Manually set the position of the text to the bottom left
                bottom: Val::Percent(2.0), 
                ..default()
            },
            ..default()
        });
    });

    // World Cam
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(10.0, 7.5, 5.0)
                .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
            ..default()
        },
        EnvironmentMapLight {
            diffuse_map: asset_server.load("environment_maps/pisa_diffuse_rgb9e5_zstd.ktx2"),
            specular_map: asset_server.load("environment_maps/pisa_specular_rgb9e5_zstd.ktx2"),
            intensity: 250.0,
        },
    ));

    // Light
    commands.spawn(DirectionalLightBundle {
        directional_light: DirectionalLight {
            shadows_enabled: true,
            ..default()
        },
        cascade_shadow_config: CascadeShadowConfigBuilder {
            num_cascades: 1,
            maximum_distance: 1.6,
            ..default()
        }
        .into(),
        ..default()
    });

    let scene_handle = asset_server.load("calculator.glb#Scene0");
    
    commands.spawn(SceneBundle {
        scene: scene_handle,
        ..default()
    })
    .insert(ColorChange)
    .insert(Interactable);
}

/// This system starts the countdown when the mouse is clicked.
fn update_screen_albedo(
    mut countdown: ResMut<Countdown>,
 ) {
    // Only start the countdown if it's not already active
    if !countdown.is_active {
        countdown.is_active = true;
        countdown.current_count = 0; // Reset the current count
        countdown.timer.reset();  // Reset the timer to start fresh
    }
}

/// This system controls ticking the timer within the countdown resource and
/// handling its state.
fn screen_albedo(
    time: Res<Time>, 
    mut countdown: ResMut<Countdown>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    children_query: Query<&Children>,
    material_query: Query<&Handle<StandardMaterial>>,
    color_change_cube_query: Query<(Entity, &Handle<Scene>), (With<ColorChange>, With<Loaded>)>,
    mut op_index: ResMut<OpIndex>,
) {
    // Only tick the timer if the countdown is active
    if countdown.is_active {
        // Tick the timer
        countdown.timer.tick(time.delta());

        // Check if the timer has finished for the current iteration
        if countdown.timer.finished() {
            // Update the albedo before we cycle color
            CurrentMeshColor::update_gltf_material_color(
                children_query,
                color_change_cube_query,
                materials,
                material_query,
                &mut op_index,
            );

            countdown.current_count += 1;
            let color_count = MeshColor::VARIANT_COUNT;
            if op_index.index == color_count {
                op_index.index = 0;
            } else {
                op_index.index += 1;
            }
            // If we've completed all iterations, stop the countdown
            if countdown.current_count >= countdown.loop_count {
                countdown.is_active = false;
            } else {
                // Otherwise, reset the timer for the next iteration
                countdown.timer.reset();
            }
        } 
    }
}

fn animate_light_direction(
    time: Res<Time>,
    mut query: Query<&mut Transform, With<DirectionalLight>>,
) {
    for mut transform in query.iter_mut() {
        transform.rotation = Quat::from_euler(
            EulerRot::ZYX,
            0.0,
            time.elapsed_seconds() * std::f32::consts::PI / 5.0,
            -std::f32::consts::FRAC_PI_4,
        );
    }
}

fn handle_asset_events(
    mut commands: Commands,
    mut events: EventReader<AssetEvent<Scene>>,
    color_change_cube_query: Query<(Entity, &Handle<Scene>), With<ColorChange>>,
) {
    for event in events.read() {
        if let AssetEvent::Added { id } = event {
            for (entity, scene_handle) in color_change_cube_query.iter() {
                if *id == scene_handle.id() {
                    commands.entity(entity).insert(Loaded);
                }
            }
        }
    }
}