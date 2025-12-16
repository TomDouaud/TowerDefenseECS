// src/enemy.rs

use bevy::prelude::*;
use crate::{
    AppState, GameAssets, GlobalPause, 
    game::{Path, PlayerStats},         
    constants::enemies as EnemyConstants
};

// --- Les Composants ---
#[derive(Component)]
pub struct Enemy {
    pub speed: f32,
}

#[derive(Component)]
pub struct Health {
    pub current: i32,
    pub max: i32,
}

#[derive(Component)]
pub struct HealthBar;

#[derive(Component)]
pub struct PathFollower {
    pub path_index: usize,
}

#[derive(Resource)]
struct EnemySpawnTimer {
    timer: Timer,
}

// --- Les Plugin ---
pub struct EnemyPlugin;

impl Plugin for EnemyPlugin {
    fn build(&self, app: &mut App) {
        app
            .insert_resource(EnemySpawnTimer {
                timer: Timer::from_seconds(1.5, TimerMode::Repeating),
            })
            .add_systems(Update, 
                (
                    spawn_enemies, 
                    move_enemies, 
                    animate_enemy_rotation, 
                    enemy_death_system,
                    update_health_bars
                )
                // Bien faire attention à ne pas faire tourner ces systèmes quand le jeu est en pause
                .run_if(in_state(AppState::Playing).and_then(not_paused))
            );
    }
}

fn not_paused(pause: Res<GlobalPause>) -> bool { !pause.0 }

// ---  LesSystèmes ---

fn spawn_enemies(
    mut commands: Commands,
    assets: Res<GameAssets>,
    path: Res<Path>,
    time: Res<Time>,
    mut spawn_timer: ResMut<EnemySpawnTimer>,
) {
    spawn_timer.timer.tick(time.delta());

    if spawn_timer.timer.just_finished() {
        if path.points.is_empty() { return; }
        
        let start_pos = path.points[0]; 
        let hp = 85; 
        let speed = 50.0; 

        commands.spawn((
            SpriteSheetBundle {
                texture: assets.sprite_atlas.clone(),
                atlas: TextureAtlas { layout: assets.sprite_atlas_layout.clone(), index: 10 },
                transform: Transform::from_xyz(start_pos.x, start_pos.y, 1.0), 
                ..default()
            },
            Enemy { speed },
            Health { current: hp, max: hp },
            PathFollower { path_index: 1 }, 
            Name::new("Orc"),
        ))
        .with_children(|parent| {
            // Fond noir
            parent.spawn(SpriteBundle {
                sprite: Sprite { color: Color::BLACK, custom_size: Some(Vec2::new(22.0, 6.0)), ..default() },
                transform: Transform::from_xyz(0.0, 20.0, 0.1), 
                ..default()
            });
            // Barre rouge
            parent.spawn((
                SpriteBundle {
                    sprite: Sprite { color: Color::RED, custom_size: Some(Vec2::new(20.0, 4.0)), ..default() },
                    transform: Transform::from_xyz(0.0, 20.0, 0.2), 
                    ..default()
                },
                HealthBar,
            ));
        });
    }
}

fn move_enemies(
    mut commands: Commands,
    mut query: Query<(Entity, &mut Transform, &Enemy, &mut PathFollower)>,
    path: Res<Path>,
    time: Res<Time>,
    mut stats: ResMut<PlayerStats>,
) {
    if path.points.is_empty() { return; }

    for (entity, mut transform, enemy, mut follower) in query.iter_mut() {
        if follower.path_index >= path.points.len() {
            // Arrivé au bout -> Dégâts au joueur
            stats.lives -= 1;
            commands.entity(entity).despawn_recursive();
            continue;
        }

        let target = path.points[follower.path_index];
        let dir = target - transform.translation.truncate();
        let dist = dir.length();
        let step = enemy.speed * time.delta_seconds();

        if dist <= step {
            transform.translation.x = target.x;
            transform.translation.y = target.y;
            follower.path_index += 1;
        } else {
            let movement = dir.normalize() * step;
            transform.translation.x += movement.x;
            transform.translation.y += movement.y;
        }
    }
}

fn animate_enemy_rotation(mut query: Query<(&mut Transform, &PathFollower)>, path: Res<Path>) {
    if path.points.is_empty() { return; }
    for (mut transform, follower) in query.iter_mut() {
        if follower.path_index < path.points.len() {
            let diff = path.points[follower.path_index] - transform.translation.truncate();
            if diff.x.abs() > diff.y.abs() {
                if diff.x > 0.0 { transform.rotation = Quat::IDENTITY; } 
                else { transform.rotation = Quat::from_rotation_y(std::f32::consts::PI); }
            } 
        }
    }
}

fn update_health_bars(
    mut bar_query: Query<(&mut Transform, &Parent, &mut Visibility), With<HealthBar>>,
    health_query: Query<&Health>,
) {
    for (mut transform, parent, mut vis) in bar_query.iter_mut() {
        if let Ok(health) = health_query.get(parent.get()) {
            // Masquer si plein
            if health.current >= health.max {
                *vis = Visibility::Hidden;
            } else {
                *vis = Visibility::Inherited;
                let percent = (health.current as f32 / health.max as f32).clamp(0.0, 1.0);
                transform.scale.x = percent;
            }
        }
    }
}

fn enemy_death_system(
    mut commands: Commands, 
    query: Query<(Entity, &Health)>,
    mut stats: ResMut<PlayerStats>,
) {
    for (entity, health) in query.iter() {
        if health.current <= 0 {
            stats.money += 5; 
            commands.entity(entity).despawn_recursive();
        }
    }
}