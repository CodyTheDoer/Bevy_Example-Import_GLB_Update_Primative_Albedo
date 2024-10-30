use bevy::{prelude::*,
    asset::{AssetEvent, Assets, Handle},
    input::common_conditions::*,
    log::info,
    pbr::{CascadeShadowConfigBuilder, DirectionalLightShadowMap},
};

fn main() {
    App::new()
        .insert_resource(DirectionalLightShadowMap { size: 4096 })
        .insert_resource(OpIndex::new())
        // .init_resource::<Countdown>()
        .init_resource::<CurrentMeshColor>()
        .add_plugins(DefaultPlugins)
        .add_systems(Startup, setup)
        .add_systems(Update, (
            animate_light_direction,
            // countdown,
            handle_asset_events,
            cycle_color_when_completed,
            CurrentMeshColor::update_gltf_material_color.run_if(input_pressed(MouseButton::Left)),
        ))
        .run();
}

// --- enum Declaration --- //
#[derive(Debug, Resource)]
enum MeshColor { // If changed update VARIANT_COUNT 
    Black,
    White,
    Red,
    Green,
    Blue,
}

// --- struct Declaration --- //
#[derive(Component)]
struct ColorChangeCube;

#[derive(Default, Resource)]
struct CurrentMeshColor;

#[derive(Component)]
struct Loaded;

#[derive(Clone, Resource)]
struct OpIndex {
    index: u32,
}

#[derive(Component, Deref, DerefMut)]
struct CycleOnCompletionTimer(Timer);

// --- fn Implementation --- //
fn setup(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
) {
    commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(10.0, 10.0, 35.0)
                .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
            ..default()
        },
        EnvironmentMapLight {
            diffuse_map: asset_server.load("environment_maps/pisa_diffuse_rgb9e5_zstd.ktx2"),
            specular_map: asset_server.load("environment_maps/pisa_specular_rgb9e5_zstd.ktx2"),
            intensity: 250.0,
        },
    ));

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

    let scene_handle = asset_server.load("cube.glb#Scene0");
    commands.spawn(SceneBundle {
        scene: scene_handle.clone(),
        ..default()
    })
    .insert(ColorChangeCube);

    // --- timer logic integration --- //

    commands.spawn(CycleOnCompletionTimer(Timer::from_seconds(
        0.25, // <----- Update seconds between triggered logic cycles here
        TimerMode::Repeating,
    )));
}

/// This system ticks the `Timer` on the entity with the `CycleOnCompletionTimer`
/// component using bevy's `Time` resource to get the delta between each update.
fn cycle_color_when_completed(
    time: Res<Time>, 
    mut query: Query<&mut CycleOnCompletionTimer>,
    mut op_index: ResMut<OpIndex>,
) {
    for mut timer in &mut query {
        if timer.tick(time.delta()).just_finished() {
            let color_count = MeshColor::VARIANT_COUNT;
            if op_index.index == color_count {
                op_index.index = 0;
            } else {
                op_index.index += 1;
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
    color_change_cube_query: Query<(Entity, &Handle<Scene>), With<ColorChangeCube>>,
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

// --- impl Implementation including sub functions --- //
impl MeshColor {
    const VARIANT_COUNT: u32 = 4;
}

impl OpIndex {
    fn new() -> Self {
        let index: u32 = 0;
        OpIndex {
            index,
        }
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
        op: &Res<OpIndex>,
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
        mut materials: ResMut<Assets<StandardMaterial>>,
        children_query: Query<&Children>,
        material_query: Query<&Handle<StandardMaterial>>,
        color_chang_cube_query: Query<(Entity, &Handle<Scene>), (With<ColorChangeCube>, With<Loaded>)>,
        op_index: Res<OpIndex>,
    ) {
        for (entity, _) in color_chang_cube_query.iter() {
            if let Ok(children) = children_query.get(entity) {
                Self::process_entity_children(
                    &mut materials,
                    &children_query,
                    &material_query,
                    children,
                    &op_index,         
                );
            }
        }
    }

    fn process_entity_children(
        materials: &mut ResMut<Assets<StandardMaterial>>,
        children_query: &Query<&Children>,
        material_query: &Query<&Handle<StandardMaterial>>,
        children: &Children,
        op_index: &Res<OpIndex>,
    ) {
        for &child in children.iter() {
            // Check if the entity has a material handle
            if let Ok(material_handle) = material_query.get(child) {
                if let Some(material) = materials.get_mut(material_handle) {
                    material.base_color = CurrentMeshColor::update_current_mesh_color(op_index);
                }
            }
    
            // Recursively check grandchildren
            if let Ok(grandchildren) = children_query.get(child) {
                Self::process_entity_children(
                    materials,
                    children_query,
                    material_query,
                    grandchildren,
                    op_index,
                );
            }
        }
    }    
}