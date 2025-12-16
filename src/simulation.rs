// src/simulation.rs

use bevy::{
    prelude::*, 
    sprite::SpriteSheetBundle,
    diagnostic::{DiagnosticsStore, FrameTimeDiagnosticsPlugin},
    ui::node_bundles::AtlasImageBundle,
};
use crate::{
    AppState, GameAssets, GlobalPause,
    level, 
    game::{Path, GameTile, TileType, get_tile_type, get_atlas_index, tower_shooting},
    tower::{Tower, TowerType},
    enemy::{Enemy, Health, HealthBar},
    projectile::Projectile,
};

// --- Composants ---
#[derive(Component)] pub struct SimComponent;
#[derive(Component)] struct SimStatsText;
#[derive(Component)] struct PauseOverlay;
#[derive(Component)] struct BtnMenu;
#[derive(Component)] struct BtnPause;
#[derive(Component)] struct SimPathFollower { path_index: usize }

#[derive(Resource)]
struct SimState {
    start_time: f64,
    total_spawned: u32,
    last_log_time: f64,
    spawn_timer: Timer, 
    finished: bool,
}

pub struct SimulationPlugin;

impl Plugin for SimulationPlugin {
    fn build(&self, app: &mut App) {
        app
            .add_systems(OnEnter(AppState::Simulation), setup_simulation)
            .add_systems(Update, (
                ui_interaction,
                update_ui_text,
                // Systèmes soumis à la pause
                (
                    simulation_logic, 
                    move_sim_enemies_loop, 
                    update_sim_visuals, // Rotation + HealthBars
                    tower_shooting,
                    cleanup_dead_sim_enemies // Mort simple (sans argent)
                ).run_if(not_paused), 
            ).run_if(in_state(AppState::Simulation)))
            .add_systems(OnExit(AppState::Simulation), cleanup_simulation);
    }
}

fn not_paused(pause: Res<GlobalPause>) -> bool { !pause.0 }

// --- Setup ---
fn setup_simulation(mut commands: Commands, assets: Res<GameAssets>, mut pause: ResMut<GlobalPause>, playing_entities: Query<Entity, With<crate::game::GameComponent>>, existing_sim: Query<Entity, With<SimComponent>>) {
    println!("=== DÉMARRAGE BENCHMARK (Mode Stress Test) ===");
    pause.0 = false;

    for e in playing_entities.iter() { commands.entity(e).despawn_recursive(); }
    for e in existing_sim.iter() { commands.entity(e).despawn_recursive(); }

    let level_data = level::get_level_data();
    const TILE_SIZE: f32 = 32.0;
    const MAP_WIDTH: f32 = 20.0 * TILE_SIZE;
    const MAP_HEIGHT: f32 = 20.0 * TILE_SIZE;
    let vertical_shift = 50.0;
    let x_offset = -MAP_WIDTH / 2.0 + TILE_SIZE / 2.0;
    let y_offset = (MAP_HEIGHT / 2.0 - TILE_SIZE / 2.0) + vertical_shift;
    let mut start_pos = Vec2::ZERO;

    // 1. Map & Tours
    for (y, row) in level_data.iter().enumerate() {
        for (x, &tile_id) in row.iter().enumerate() {
            let pos = Vec2::new(x_offset + x as f32 * TILE_SIZE, y_offset - y as f32 * TILE_SIZE);
            let tile_type = get_tile_type(tile_id);
            
            let mut spawn_tile = |idx: usize, rot: Quat, z: f32| {
                commands.spawn((
                    SpriteSheetBundle {
                        texture: assets.sprite_atlas.clone(),
                        atlas: TextureAtlas { layout: assets.sprite_atlas_layout.clone(), index: idx },
                        transform: Transform { translation: pos.extend(z), rotation: rot, ..default() },
                        ..default()
                    }, SimComponent,
                ));
            };

            match tile_id {
                8..=19 => { spawn_tile(get_atlas_index(0,0), Quat::IDENTITY, 0.0); let (i,r) = get_composite_info(tile_id); spawn_tile(i,r,0.1); },
                20|21 => { spawn_tile(get_atlas_index(8,0), Quat::IDENTITY, 0.0); let i = if tile_id==20 { get_atlas_index(7,2) } else { get_atlas_index(8,2) }; spawn_tile(i, Quat::IDENTITY, 0.1); },
                _ => { let (i,r) = get_simple_tile_info(tile_id); spawn_tile(i,r,0.0); }
            }
            if tile_id == 20 { start_pos = pos; }

            if tile_type == TileType::Grass {
                let tower_type = determine_sim_tower_type(x, y, level_data);
                let (range, damage, cooldown) = tower_type.get_sim_stats();
                commands.spawn((
                    SpriteSheetBundle {
                        texture: assets.sprite_atlas.clone(),
                        atlas: TextureAtlas { layout: assets.sprite_atlas_layout.clone(), index: tower_type.get_sprite_index() },
                        transform: Transform::from_xyz(pos.x, pos.y, 2.0),
                        ..default()
                    },
                    Tower { range, damage, cooldown: Timer::from_seconds(cooldown, TimerMode::Repeating) },
                    SimComponent,
                ));
            }
        }
    }

    // 2. Pathfinding
    let mut path_points = Vec::new();
    path_points.push(start_pos);
    let mut grid_x = ((start_pos.x - x_offset) / TILE_SIZE).round() as i32;
    let mut grid_y = ((y_offset - start_pos.y) / TILE_SIZE).round() as i32;
    let mut last_grid_pos = (grid_x, grid_y);
    for _ in 0..100 {
        let mut found_next = false;
        let neighbors = [(0, -1), (0, 1), (-1, 0), (1, 0)];
        for (dx, dy) in neighbors {
            let nx = grid_x + dx;
            let ny = grid_y + dy;
            if nx < 0 || ny < 0 || nx >= 20 || ny >= 20 { continue; }
            if (nx, ny) == last_grid_pos { continue; }
            let tid = level_data[ny as usize][nx as usize];
            let ttype = get_tile_type(tid);
            if ttype == TileType::Road || ttype == TileType::End {
                last_grid_pos = (grid_x, grid_y);
                grid_x = nx;
                grid_y = ny;
                path_points.push(Vec2::new(x_offset + nx as f32 * TILE_SIZE, y_offset - ny as f32 * TILE_SIZE));
                found_next = true;
                if ttype == TileType::End { break; }
                break;
            }
        }
        if !found_next { break; }
        if get_tile_type(level_data[grid_y as usize][grid_x as usize]) == TileType::End { break; }
    }
    commands.insert_resource(Path { points: path_points });

    // 3. UI & State
    let bar_color = Color::rgb_u8(220, 123, 15);
    commands.spawn((
        NodeBundle {
            style: Style { width: Val::Percent(100.0), height: Val::Px(100.0), position_type: PositionType::Absolute, bottom: Val::Px(0.0), align_items: AlignItems::Center, padding: UiRect::all(Val::Px(20.0)), ..default() },
            background_color: bar_color.into(), ..default()
        }, SimComponent,
    )).with_children(|parent| {
        spawn_ui_button(parent, "MENU", BtnMenu);
        spawn_ui_button(parent, "PAUSE", BtnPause);
        parent.spawn((TextBundle::from_section("Init...", TextStyle { font_size: 20.0, color: Color::BLACK, ..default() }).with_style(Style { margin: UiRect::left(Val::Px(50.0)), ..default() }), SimStatsText));
    });

    commands.spawn((
        NodeBundle {
            style: Style { width: Val::Percent(100.0), height: Val::Percent(100.0), position_type: PositionType::Absolute, justify_content: JustifyContent::Center, align_items: AlignItems::Center, display: Display::None, ..default() },
            background_color: Color::rgba(0.0, 0.0, 0.0, 0.5).into(), z_index: ZIndex::Global(100), ..default()
        }, PauseOverlay, SimComponent,
    )).with_children(|parent| {
        parent.spawn(TextBundle::from_section("SIMULATION PAUSE", TextStyle { font_size: 50.0, color: Color::WHITE, ..default() }));
    });

    commands.insert_resource(SimState {
        start_time: 0.0, total_spawned: 0, last_log_time: 0.0,
        spawn_timer: Timer::from_seconds(1.0 / 60.0, TimerMode::Repeating),
        finished: false,
    });
}

// --- Logic ---
fn simulation_logic(mut commands: Commands, assets: Res<GameAssets>, mut sim_state: ResMut<SimState>, path: Res<Path>, time: Res<Time>) {
    if sim_state.start_time == 0.0 { sim_state.start_time = time.elapsed_seconds_f64(); }
    if sim_state.finished || path.points.is_empty() { return; }
    let elapsed = time.elapsed_seconds_f64() - sim_state.start_time;
    if elapsed >= 5.0 * 60.0 { println!("FIN DE LA SIMULATION"); sim_state.finished = true; return; }

    sim_state.spawn_timer.tick(time.delta());
    let ticks = sim_state.spawn_timer.times_finished_this_tick();
    if ticks > 0 {
        let start_pos = path.points[0];
        let to_spawn = ticks * 10; 
        sim_state.total_spawned += to_spawn;
        for _ in 0..to_spawn {
            commands.spawn((
                SpriteSheetBundle { texture: assets.sprite_atlas.clone(), atlas: TextureAtlas { layout: assets.sprite_atlas_layout.clone(), index: 10 }, transform: Transform::from_xyz(start_pos.x, start_pos.y, 1.0), ..default() },
                Enemy { speed: 50.0 }, Health { current: 85, max: 85 }, SimPathFollower { path_index: 1 }, SimComponent,
            )).with_children(|parent| {
                parent.spawn(SpriteBundle { 
                    sprite: Sprite { color: Color::BLACK, custom_size: Some(Vec2::new(22.0, 6.0)), ..default() }, 
                    transform: Transform::from_xyz(0.0, 20.0, 0.1), 
                    visibility: Visibility::Hidden, // CACHÉ PAR DÉFAUT
                    ..default() 
                });
                parent.spawn((SpriteBundle { 
                    sprite: Sprite { color: Color::RED, custom_size: Some(Vec2::new(20.0, 4.0)), ..default() }, 
                    transform: Transform::from_xyz(0.0, 20.0, 0.2), 
                    visibility: Visibility::Hidden, // CACHÉ PAR DÉFAUT
                    ..default() 
                }, HealthBar));
            });
        }
    }
}

fn move_sim_enemies_loop(mut query: Query<(&mut Transform, &Enemy, &mut SimPathFollower)>, path: Res<Path>, time: Res<Time>) {
    if path.points.is_empty() { return; }
    for (mut transform, enemy, mut follower) in query.iter_mut() {
        if follower.path_index >= path.points.len() {
            follower.path_index = 1;
            let start = path.points[0];
            transform.translation.x = start.x;
            transform.translation.y = start.y;
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
            let mv = dir.normalize() * step;
            transform.translation.x += mv.x;
            transform.translation.y += mv.y;
        }
    }
}

// Système visuel pour la simulation (Rotation + HP)
fn update_sim_visuals(
    mut enemies: Query<(&mut Transform, &SimPathFollower), With<Enemy>>,
    mut bars: Query<(&mut Transform, &Parent, &mut Visibility), (With<HealthBar>, Without<Enemy>)>,
    health_q: Query<&Health>,
    path: Res<Path>
) {
    if !path.points.is_empty() {
        for (mut t, f) in enemies.iter_mut() {
            if f.path_index < path.points.len() {
                let diff = path.points[f.path_index] - t.translation.truncate();
                if diff.x.abs() > diff.y.abs() {
                    if diff.x > 0.0 { t.rotation = Quat::IDENTITY; } else { t.rotation = Quat::from_rotation_y(std::f32::consts::PI); }
                } 
            }
        }
    }
    
    for (mut t, parent, mut vis) in bars.iter_mut() {
        if let Ok(h) = health_q.get(parent.get()) {
            // Si 100% HP -> Caché
            if h.current >= h.max {
                *vis = Visibility::Hidden;
            } else {
                *vis = Visibility::Inherited; // Affiche si blessé
                if h.max > 0 { 
                    t.scale.x = (h.current as f32 / h.max as f32).clamp(0.0, 1.0); 
                }
            }
        }
    }
}

fn cleanup_dead_sim_enemies(
    mut commands: Commands, 
    query: Query<(Entity, &Health), With<SimComponent>>
) {
    for (entity, health) in query.iter() {
        if health.current <= 0 {
            // Pas de log pour ne pas spammer la console en benchmark, mais ça marche
            commands.entity(entity).despawn_recursive();
        }
    }
}

// --- UI ---
fn ui_interaction(mut cmd: Commands, mut sim: ResMut<SimState>, mut q: Query<(&Interaction, Option<&BtnMenu>, Option<&BtnPause>), (Changed<Interaction>, With<Button>)>, mut next: ResMut<NextState<AppState>>, mut pause: ResMut<GlobalPause>) {
    for (int, btn_menu, btn_pause) in q.iter_mut() {
        if *int == Interaction::Pressed {
            if btn_menu.is_some() { pause.0 = false; next.set(AppState::Menu); }
            else if btn_pause.is_some() { pause.0 = !pause.0; }
        }
    }
}

fn update_ui_text(time: Res<Time>, diag: Res<DiagnosticsStore>, sim: Res<SimState>, enemies: Query<Entity, With<Enemy>>, mut txt: Query<&mut Text, With<SimStatsText>>, mut btn: Query<&mut Text, (With<BtnPause>, Without<SimStatsText>)>, mut over: Query<&mut Style, With<PauseOverlay>>, pause: Res<GlobalPause>) {
    let fps = diag.get(&FrameTimeDiagnosticsPlugin::FPS).and_then(|f| f.smoothed()).unwrap_or(0.0);
    let count = enemies.iter().count();
    let elapsed = time.elapsed_seconds_f64() - sim.start_time;
    let mins = (elapsed / 60.0) as u32;
    let secs = (elapsed % 60.0) as u32;

    for mut t in txt.iter_mut() {
        t.sections[0].value = format!("Temps: {:02}:{:02} / 05:00   Total Spawnés: {}   Actifs: {}\nFPS: {:.1}", mins, secs, sim.total_spawned, count, fps);
    }
    for mut t in btn.iter_mut() { t.sections[0].value = if pause.0 { "RESUME".into() } else { "PAUSE".into() }; }
    for mut s in over.iter_mut() { s.display = if pause.0 { Display::Flex } else { Display::None }; }
}

fn cleanup_simulation(
    mut commands: Commands, 
    query: Query<Entity, With<SimComponent>>,
    // On nettoie aussi les projectiles qui traînent
    projectiles: Query<Entity, With<Projectile>>,
) {
    for entity in query.iter() {
        commands.entity(entity).despawn_recursive();
    }
    for entity in projectiles.iter() {
        commands.entity(entity).despawn_recursive();
    }
    commands.remove_resource::<Path>();
    commands.remove_resource::<SimState>();
}

// --- Helpers ---
fn spawn_ui_button<T: Component>(parent: &mut ChildBuilder, text: &str, marker: T) {
    parent.spawn((ButtonBundle { style: Style { width: Val::Px(80.0), height: Val::Px(30.0), margin: UiRect::right(Val::Px(10.0)), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() }, background_color: Color::rgb(0.33, 0.23, 0.15).into(), border_color: BorderColor(Color::BLACK), ..default() }, marker)).with_children(|p| { p.spawn(TextBundle::from_section(text, TextStyle { font_size: 16.0, color: Color::rgb(0.9, 0.85, 0.7), ..default() })); });
}
fn determine_sim_tower_type(x: usize, y: usize, level: &[[u32; 20]; 20]) -> TowerType {
    let neighbors = [(0, -1), (0, 1), (-1, 0), (1, 0)];
    let mut next_to_road = false;
    for (dx, dy) in neighbors {
        let nx = x as i32 + dx; let ny = y as i32 + dy;
        if nx >= 0 && ny >= 0 && nx < 20 && ny < 20 {
            let tid = level[ny as usize][nx as usize];
            let ttype = get_tile_type(tid);
            if ttype == TileType::Road || ttype == TileType::Start || ttype == TileType::End { next_to_road = true; break; }
        }
    }
    if next_to_road { return TowerType::Canon; }
    TowerType::Archer
}
fn get_composite_info(id: u32) -> (usize, Quat) {
    let r90 = -90.0f32.to_radians(); let r180 = 180.0f32.to_radians(); let r270 = 90.0f32.to_radians();
    match id {
        8 => (get_atlas_index(5, 0), Quat::IDENTITY), 9 => (get_atlas_index(5, 0), Quat::from_rotation_z(r90)), 10 => (get_atlas_index(5, 0), Quat::from_rotation_z(r180)), 11 => (get_atlas_index(5, 0), Quat::from_rotation_z(r270)), 12 => (get_atlas_index(6, 0), Quat::IDENTITY), 13 => (get_atlas_index(6, 0), Quat::from_rotation_z(r90)), 14 => (get_atlas_index(6, 0), Quat::from_rotation_z(r180)), 15 => (get_atlas_index(6, 0), Quat::from_rotation_z(r270)), 16 => (get_atlas_index(4, 0), Quat::IDENTITY), 17 => (get_atlas_index(4, 0), Quat::from_rotation_z(r90)), 18 => (get_atlas_index(4, 0), Quat::from_rotation_z(r180)), 19 => (get_atlas_index(4, 0), Quat::from_rotation_z(r270)), _ => (0, Quat::IDENTITY),
    }
}
fn get_simple_tile_info(id: u32) -> (usize, Quat) {
    let r90 = -90.0f32.to_radians(); let r180 = 180.0f32.to_radians(); let r270 = 90.0f32.to_radians();
    match id {
        0 => (get_atlas_index(9, 0), Quat::IDENTITY), 1 => (get_atlas_index(0, 0), Quat::IDENTITY), 2 => (get_atlas_index(8, 0), Quat::IDENTITY), 3 => (get_atlas_index(8, 0), Quat::from_rotation_z(r90)), 4 => (get_atlas_index(7, 0), Quat::IDENTITY), 5 => (get_atlas_index(7, 0), Quat::from_rotation_z(r90)), 6 => (get_atlas_index(7, 0), Quat::from_rotation_z(r180)), 7 => (get_atlas_index(7, 0), Quat::from_rotation_z(r270)), _ => (get_atlas_index(0, 0), Quat::IDENTITY),
    }
}