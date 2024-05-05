use bevy::{
    prelude::*,
    transform::TransformSystem,
    window::{CursorGrabMode, PrimaryWindow},
};
use big_space::{
    camera::{CameraController, CameraInput},
    propagation::IgnoreFloatingOrigin,
    world_query::GridTransformReadOnly,
    FloatingOrigin, GridCell,
};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.build().disable::<TransformPlugin>(),
            bevy_framepace::FramepacePlugin,
            DemoPlugin::<1>,
            DemoPlugin::<2>,
            DemoPlugin::<3>,
            DemoPlugin::<4>,
        ))
        .insert_resource(ClearColor(Color::BLACK))

        .run()
}

struct DemoPlugin<const L: u8>;

impl<const L: u8> Plugin for DemoPlugin<L> {
    fn build(&self, app: &mut App) {
        app.add_plugins((
            big_space::FloatingOriginPlugin::<i128, L>::default(),
            big_space::debug::FloatingOriginDebugPlugin::<i128,L>::default(),
            big_space::camera::CameraControllerPlugin::<i128,L>::default(),
        ))        .add_systems(Startup, (setup::<L>, ui_setup::<L>))
        .add_systems(Update, ui_text_system::<L>)
        // .add_systems(Update, animate_camera::<L>)
;
    }
}

fn ignore_all_floating_origins(commands: &mut Commands, ent_id: Entity) {
    commands.entity(ent_id).insert(    (
        IgnoreFloatingOrigin::<1>,
        IgnoreFloatingOrigin::<2>,
        IgnoreFloatingOrigin::<3>,
        IgnoreFloatingOrigin::<4>,
    ));
}

fn ignore_other_floating_origins<const L: u8>(commands: &mut Commands, ent_id: Entity) {
    if L != 1 {
        commands.entity(ent_id).insert(    (
            IgnoreFloatingOrigin::<1>,
        ));
    }
    if L != 2 {
        commands.entity(ent_id).insert(    (
            IgnoreFloatingOrigin::<2>,
        ));
    }
    if L != 3 {
        commands.entity(ent_id).insert(    (
            IgnoreFloatingOrigin::<3>,
        ));
    }
    if L != 4 {
        commands.entity(ent_id).insert(    (
            IgnoreFloatingOrigin::<4>,
        ));
    }
}

fn setup<const L: u8>(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // camera
    let cam_id = commands.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 0.0, 8.0)
                .looking_at(Vec3::new(0.0, 0.0, 0.0), Vec3::Y),
            projection: Projection::Perspective(PerspectiveProjection {
                near: 1e-18,
                ..default()
            }),
            camera: Camera {
                order: L as isize,
                ..default()
            },
            
            ..default()
        },
        GridCell::<i128,L>::default(), // All spatial entities need this component
        FloatingOrigin::<L>,              // Important: marks the floating origin entity for rendering.
        CameraController::<L>::default() // Built-in camera controller
            .with_speed_bounds([10e-18, 10e35])
            .with_smoothness(0.9, 0.8)
            .with_speed(1.0),
    )).id();
    ignore_other_floating_origins::<L>(&mut commands, cam_id);

    let mesh_handle = meshes.add(Sphere::new(0.5).mesh().ico(32).unwrap());
    let matl_handle = materials.add(StandardMaterial {
        base_color: Color::BLUE,
        perceptual_roughness: 0.8,
        reflectance: 1.0,
        ..default()
    });

    let mut translation = Vec3::ZERO;
    for i in -16..=27 {
        let j = 10_f32.powf(i as f32);
        let k = 10_f32.powf((i - 1) as f32);
        translation.x += j / 2.0 + k;
        let item_id =commands.spawn((
            PbrBundle {
                mesh: mesh_handle.clone(),
                material: matl_handle.clone(),
                transform: Transform::from_scale(Vec3::splat(j)).with_translation(translation),
                ..default()
            },
            GridCell::<i128,L>::default(),
        )).id();
        ignore_other_floating_origins::<L>(&mut commands, item_id);
    }

    // light
    commands.spawn((DirectionalLightBundle {
        directional_light: DirectionalLight {
            illuminance: 100_000.0,
            ..default()
        },
        ..default()
    },));
}

#[derive(Component, Reflect)]
pub struct BigSpaceDebugText<const L: u8>;

#[derive(Component, Reflect)]
pub struct FunFactText<const L: u8>;

fn ui_setup<const L: u8>(mut commands: Commands) {
    let txt_id = commands.spawn((
        TextBundle::from_section(
            "",
            TextStyle {
                font_size: 28.0,
                color: Color::WHITE,
                ..default()
            },
        )
        .with_text_justify(JustifyText::Left)
        .with_style(Style {
            position_type: PositionType::Absolute,
            top: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        }),
        BigSpaceDebugText::<L>,
        // IgnoreFloatingOrigin::<0>,
    )).id();
    ignore_all_floating_origins(&mut commands, txt_id);

    let txt_id2 = commands.spawn((
        TextBundle::from_section(
            "",
            TextStyle {
                font_size: 52.0,
                color: Color::WHITE,
                ..default()
            },
        )
        .with_style(Style {
            position_type: PositionType::Absolute,
            bottom: Val::Px(10.0),
            right: Val::Px(10.0),
            left: Val::Px(10.0),
            ..default()
        })
        .with_text_justify(JustifyText::Center),
        FunFactText::<L>,
        // IgnoreFloatingOrigin::<0>,
    )).id();
    ignore_all_floating_origins(&mut commands, txt_id2);
}


#[allow(clippy::type_complexity)]
fn ui_text_system<const L: u8>(
    mut debug_text: Query<
        (&mut Text, &GlobalTransform),
        (With<BigSpaceDebugText::<L>>, Without<FunFactText<L>>),
    >,
    mut fun_text: Query<&mut Text, (With<FunFactText<L>>, Without<BigSpaceDebugText<L>>)>,
    time: Res<Time>,
    origin: Query<GridTransformReadOnly<i128, L>, With<FloatingOrigin::<L>>>,
    camera: Query<&CameraController::<L>>,
    objects: Query<&Transform, With<Handle<Mesh>>>,
) {
    let origin = origin.single();
    let translation = origin.transform.translation;

    let grid_text = format!(
        "Origin #{}: GridCell: {}x, {}y, {}z",
        L, origin.cell.x, origin.cell.y, origin.cell.z
    );

    let translation_text = format!(
        "Transform: {:>8.2}x, {:>8.2}y, {:>8.2}z",
        translation.x, translation.y, translation.z
    );

    let velocity = camera.single().velocity();
    let speed = velocity.0.length() / time.delta_seconds_f64();
    let camera_text = if speed > 3.0e8 {
        format!("Speed: {:.0e} * speed of light", speed / 3.0e8)
    } else {
        format!("Speed: {:.2e} m/s", speed)
    };

    let (nearest_text, fact_text) = if let Some(nearest) = camera.single().nearest_object() {
        let dia = objects.get(nearest.0).unwrap().scale.max_element();
        let (fact_dia, fact) = closest(dia);
        let dist = nearest.1;
        let multiple = dia / fact_dia;
        (
            format!(
                "\nNearest sphere distance: {dist:.0e} m\nNearest sphere diameter: {dia:.0e} m",
            ),
            format!("{multiple:.1}x {fact}"),
        )
    } else {
        ("".into(), "".into())
    };

    let mut debug_text = debug_text.single_mut();

    debug_text.0.sections[0].value =
        format!("{grid_text}\n{translation_text}\n{camera_text}\n{nearest_text}");

    fun_text.single_mut().sections[0].value = fact_text
}

fn closest<'a>(diameter: f32) -> (f32, &'a str) {
    let items = vec![
        (8.8e26, "diameter of the observable universe"),
        (9e25, "length of the Hercules-Corona Borealis Great Wall"),
        (1e24, "diameter of the Local Supercluster"),
        (9e22, "diameter of the Local Group"),
        (1e21, "diameter of the Milky Way galaxy"),
        (5e16, "length of the Pillars of Creation"),
        (1.8e14, "diameter of Messier 87"),
        (7e12, "diameter of Pluto's orbit"),
        (24e9, "diameter of Sagittarius A"),
        (1.4e9, "diameter of the Sun"),
        (1.4e8, "diameter of Jupiter"),
        (12e6, "diameter of Earth"),
        (3e6, "diameter of the Moon"),
        (9e3, "height of Mt. Everest"),
        (3.8e2, "height of the Empire State Building"),
        (2.5e1, "length of a train car"),
        (1.8, "height of a human"),
        (1e-1, "size of a cat"),
        (1e-2, "size of a mouse"),
        (1e-3, "size of an insect"),
        (1e-4, "diameter of a eukaryotic cell"),
        (1e-5, "width of a human hair"),
        (1e-6, "diameter of a bacteria"),
        (5e-8, "size of a phage"),
        (5e-9, "size of a transistor"),
        (1e-10, "diameter of a carbon atom"),
        (4e-11, "diameter of a hydrogen atom"),
        (4e-12, "diameter of an electron"),
        (1.9e-15, "diameter of a proton"),
    ];

    let mut min = items[0];
    for item in items.iter() {
        if (item.0 - diameter).abs() < (min.0 - diameter).abs() {
            min = item.to_owned();
        }
    }
    min
}
