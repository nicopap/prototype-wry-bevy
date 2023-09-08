//! This example displays each link to the bevy source code as a bouncing bevy-ball.

use bevy::prelude::*;
use rand::{prelude::SliceRandom, Rng};

use crate::bridge;

pub struct LinksPlugin;
impl Plugin for LinksPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<SelectionState>()
            .init_resource::<BevyIcon>()
            .init_resource::<LinkSelection>()
            .add_event::<NewPage>()
            .add_systems(Startup, setup)
            .add_systems(
                Update,
                (
                    spawn_links,
                    velocity,
                    move_system,
                    collision,
                    select_system,
                    navigate,
                ),
            );
    }
}

#[derive(Event, Debug)]
pub struct NewPage {
    pub links: Vec<String>,
}

#[derive(Resource, Default)]
struct LinkSelection {
    order: Vec<Entity>,
    idx: usize,
}

#[derive(Resource)]
struct SelectionState {
    timer: Timer,
    has_triggered: bool,
}

impl Default for SelectionState {
    fn default() -> Self {
        Self {
            timer: Timer::from_seconds(SHOWCASE_TIMER_SECS, TimerMode::Repeating),
            has_triggered: false,
        }
    }
}

#[derive(Component)]
struct LinkDisplay;

#[derive(Component)]
struct Link {
    target: String,
    hue: f32,
}

#[derive(Component)]
struct Velocity {
    translation: Vec3,
    rotation: f32,
}

const GRAVITY: f32 = 9.821 * 100.0;
const SPRITE_SIZE: f32 = 75.0;

const SATURATION_DESELECTED: f32 = 0.3;
const LIGHTNESS_DESELECTED: f32 = 0.2;
const SATURATION_SELECTED: f32 = 0.9;
const LIGHTNESS_SELECTED: f32 = 0.7;
const ALPHA: f32 = 0.92;

const SHOWCASE_TIMER_SECS: f32 = 1.2;

#[derive(Resource)]
struct BevyIcon(Handle<Image>);
impl FromWorld for BevyIcon {
    fn from_world(world: &mut World) -> Self {
        let assets = world.resource::<AssetServer>();
        BevyIcon(assets.load("bevy_icon.png"))
    }
}

fn spawn_links(
    mut new_page: EventReader<NewPage>,
    mut commands: Commands,
    icon: Res<BevyIcon>,
    mut link_selection: ResMut<LinkSelection>,
    pre_existing_links: Query<Entity, With<Link>>,
) {
    let Some(new_page) = new_page.iter().next() else {
        return;
    };
    info!(
        "Got NewPage request, spawning {} links",
        new_page.links.len()
    );
    let mut rng = rand::thread_rng();
    for entity in &pre_existing_links {
        commands.entity(entity).despawn();
    }
    link_selection.order.clear();
    for name in &new_page.links {
        let pos = (rng.gen_range(-400.0..400.0), rng.gen_range(0.0..400.0));
        let dir = rng.gen_range(-1.0..1.0);
        let velocity = Vec3::new(dir * 500.0, 0.0, 0.0);
        let hue = rng.gen_range(0.0..=360.0);
        let transform = Transform::from_xyz(pos.0, pos.1, 0.0);
        let target = name.clone();
        let texture = icon.0.clone();
        let sprite = Sprite {
            custom_size: Some(Vec2::new(1.0, 1.0) * SPRITE_SIZE),
            color: Color::hsla(hue, SATURATION_DESELECTED, LIGHTNESS_DESELECTED, ALPHA),
            flip_x: rng.gen_bool(0.5),
            ..default()
        };
        let entity = commands.spawn((
            Link { target, hue },
            Velocity { translation: velocity, rotation: -dir * 5.0 },
            SpriteBundle { sprite, texture, transform, ..default() },
        ));
        link_selection.order.push(entity.id());
    }
    link_selection.order.shuffle(&mut rng);
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());

    commands.spawn((
        TextBundle::from_sections([
            TextSection::new(
                "Links in current webpage",
                TextStyle { font_size: 60.0, color: Color::WHITE, ..default() },
            ),
            TextSection::from_style(TextStyle {
                font_size: 60.0,
                color: Color::WHITE,
                ..default()
            }),
        ])
        .with_style(Style { align_self: AlignSelf::FlexEnd, ..default() }),
        LinkDisplay,
    ));
}

/// Finds the next link to display and selects the entity
fn select_system(
    mut timer: ResMut<SelectionState>,
    mut link_selection: ResMut<LinkSelection>,
    mut text_query: Query<&mut Text, With<LinkDisplay>>,
    mut query: Query<(&Link, &mut Sprite, &mut Transform)>,
    time: Res<Time>,
) {
    let Some(&entity) = link_selection.order.get(link_selection.idx) else {
        return;
    };
    if !timer.timer.tick(time.delta()).just_finished() {
        return;
    }
    if !timer.has_triggered {
        let mut text = text_query.single_mut();
        text.sections[0].value = "Link: ".to_string();

        timer.has_triggered = true;
    }
    if let Ok((link, mut sprite, mut transform)) = query.get_mut(entity) {
        deselect(&mut sprite, link, &mut transform);
    }

    if (link_selection.idx + 1) < link_selection.order.len() {
        link_selection.idx += 1;
    } else {
        link_selection.idx = 0;
    }

    let entity = link_selection.order[link_selection.idx];

    if let Ok((link, mut sprite, mut transform)) = query.get_mut(entity) {
        let mut text = text_query.single_mut();
        select(&mut sprite, link, &mut transform, &mut text);
    }
}

/// Change the tint color to the "selected" color, bring the object to the front
/// and display the name.
fn select(sprite: &mut Sprite, link: &Link, transform: &mut Transform, text: &mut Text) {
    sprite.color = Color::hsla(link.hue, SATURATION_SELECTED, LIGHTNESS_SELECTED, ALPHA);

    transform.translation.z = 100.0;

    text.sections[1].value.clone_from(&link.target);
    text.sections[1].style.color = sprite.color;
}

/// Change the modulate color to the "deselected" color and push
/// the object to the back.
fn deselect(sprite: &mut Sprite, link: &Link, transform: &mut Transform) {
    sprite.color = Color::hsla(link.hue, SATURATION_DESELECTED, LIGHTNESS_DESELECTED, ALPHA);

    transform.translation.z = 0.0;
}

fn navigate(
    input: Res<Input<KeyCode>>,
    mut events: EventWriter<bridge::Event>,
    link_selection: Res<LinkSelection>,
    query: Query<&Link>,
) {
    if !input.just_pressed(KeyCode::Space) {
        return;
    }
    info!("Sending navigation request, user pressed space");
    let entity = link_selection.order[link_selection.idx];
    if let Ok(link) = query.get(entity) {
        let target = link.target.clone();
        events.send(bridge::Event::NavigateToPage(target));
    }
}
/// Applies gravity to all entities with velocity
fn velocity(time: Res<Time>, mut velocity_query: Query<&mut Velocity>) {
    let delta = time.delta_seconds();

    for mut velocity in &mut velocity_query {
        velocity.translation.y -= GRAVITY * delta;
    }
}

/// Checks for collisions of link-birds.
///
/// On collision with left-or-right wall it resets the horizontal
/// velocity. On collision with the ground it applies an upwards
/// force.
fn collision(
    windows: Query<&Window>,
    mut query: Query<(&mut Velocity, &mut Transform), With<Link>>,
) {
    let Ok(window) = windows.get_single() else {
        return;
    };
    let ceiling = window.height() / 2.;
    let ground = -window.height() / 2.;

    let wall_left = -window.width() / 2.;
    let wall_right = window.width() / 2.;

    // The maximum height the birbs should try to reach is one birb below the top of the window.
    let max_bounce_height = (window.height() - SPRITE_SIZE * 2.0).max(0.0);

    let mut rng = rand::thread_rng();

    for (mut velocity, mut transform) in &mut query {
        let left = transform.translation.x - SPRITE_SIZE / 2.0;
        let right = transform.translation.x + SPRITE_SIZE / 2.0;
        let top = transform.translation.y + SPRITE_SIZE / 2.0;
        let bottom = transform.translation.y - SPRITE_SIZE / 2.0;

        // clamp the translation to not go out of the bounds
        if bottom < ground {
            transform.translation.y = ground + SPRITE_SIZE / 2.0;

            // How high this birb will bounce.
            let bounce_height = rng.gen_range((max_bounce_height * 0.4)..=max_bounce_height);

            // Apply the velocity that would bounce the birb up to bounce_height.
            velocity.translation.y = (bounce_height * GRAVITY * 2.).sqrt();
        }
        if top > ceiling {
            transform.translation.y = ceiling - SPRITE_SIZE / 2.0;
            velocity.translation.y *= -1.0;
        }
        // on side walls flip the horizontal velocity
        if left < wall_left {
            transform.translation.x = wall_left + SPRITE_SIZE / 2.0;
            velocity.translation.x *= -1.0;
            velocity.rotation *= -1.0;
        }
        if right > wall_right {
            transform.translation.x = wall_right - SPRITE_SIZE / 2.0;
            velocity.translation.x *= -1.0;
            velocity.rotation *= -1.0;
        }
    }
}

/// Apply velocity to positions and rotations.
fn move_system(time: Res<Time>, mut query: Query<(&Velocity, &mut Transform)>) {
    let delta = time.delta_seconds();

    for (velocity, mut transform) in &mut query {
        transform.translation += delta * velocity.translation;
        transform.rotate_z(velocity.rotation * delta);
    }
}
