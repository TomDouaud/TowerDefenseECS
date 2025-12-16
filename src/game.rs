use bevy::{prelude::*, sprite::SpriteSheetBundle, window::PrimaryWindow, ui::node_bundles::AtlasImageBundle,};
use crate::{
    AppState, 
    GameAssets,
    GlobalPause, 
    level,
    constants::tiles as TileTypes,
    tower::{Tower, TowerType},
    enemy::Enemy,
    projectile::Projectile,
};


// Composant pour tout ce qui est dans le jeu
#[derive(Component)]
pub struct GameComponent;

// Équivalent de TileType en Java
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TileType {
    Water,
    Grass,
    Road,
    Start,
    End,
}

// Composant qui sera ajouté à chaque entité "Tuile"
#[derive(Component)]
pub struct GameTile {
    pub tile_type: TileType,
}

// Ressource pour stocker les points de départ et de fin
#[derive(Resource)]
pub struct Path {
    pub points: Vec<Vec2>, // Liste des points du chemin
}

// Ressource pour la tour sélectionnée dans le menu
#[derive(Resource, Default)]
struct SelectedTower {
    tower_type: Option<TowerType>,
}

// État local du jeu (si le jeu est en Pause ou pas)
#[derive(Resource, Default)]
struct GameState { pub is_paused: bool }

#[derive(Resource)]
pub struct PlayerStats {
    pub money: i32,
    pub lives: i32,
}

// Pour les boutons de sélection de tours
#[derive(Component)]
struct TowerButton {
    tower_type: TowerType,
}

#[derive(Component)]
struct MoneyText;

#[derive(Component)]
struct LivesText;


// Composants UI
#[derive(Component)]
struct BtnMenu;
#[derive(Component)]
struct BtnPause;
#[derive(Component)]
struct PauseOverlay;


// Équivalent de "Playing.java"
pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app
            // On initialise au démarrage, mais setup_game le fera aussi
            .init_resource::<SelectedTower>()
            .insert_resource(PlayerStats { money: 300, lives: 3 })
            .add_systems(OnEnter(AppState::Playing), (setup_game, setup_game_ui))
            .add_systems(Update, (
                tower_button_interaction, 
                grid_click_interaction, 
                ui_button_interaction,
                update_ui_text,
            ).run_if(in_state(AppState::Playing)))
            .add_systems(Update, tower_shooting.run_if(in_state(AppState::Playing).and_then(not_paused))) 
            .add_systems(OnExit(AppState::Playing), cleanup_game);
    }
}

fn not_paused(pause: Res<GlobalPause>) -> bool { !pause.0 }


pub fn get_atlas_index(x: usize, y: usize) -> usize {
    y * 10 + x
}

// équivalent constructeur Playing()
fn setup_game(
    mut commands: Commands, 
    assets: Res<GameAssets>, 
    mut pause: ResMut<GlobalPause>,
    sim_entities: Query<Entity, With<crate::simulation::SimComponent>>,
) {
    println!("Lancement du jeu (Playing) !");

    pause.0 = false;

    commands.insert_resource(PlayerStats { money: 300, lives: 3 });
    commands.init_resource::<SelectedTower>();

    let level_data = level::get_level_data();
    
    // Positions temporaires pour le pathfinding
    let mut start_pos = Vec2::ZERO;
    
    // Configuration de la grille
    const TILE_SIZE: f32 = 32.0;
    const MAP_WIDTH: f32 = 20.0 * TILE_SIZE;
    const MAP_HEIGHT: f32 = 20.0 * TILE_SIZE;
    
    // Prise en compte de l'UI en bas pour le décalage vertical
    let vertical_shift = 50.0;

    let x_offset = -MAP_WIDTH / 2.0 + TILE_SIZE / 2.0;
    // ajout du vertical_shift ici
    let y_offset = (MAP_HEIGHT / 2.0 - TILE_SIZE / 2.0) + vertical_shift;

    for (y, row) in level_data.iter().enumerate() {
        for (x, &tile_id) in row.iter().enumerate() {
            
            // Position dans le monde
            let pos = Vec2::new(
                x_offset + x as f32 * TILE_SIZE,
                y_offset - y as f32 * TILE_SIZE,
            );
            
            let tile_type = get_tile_type(tile_id);
            
            let mut rotation = Quat::IDENTITY; 
            let mut index = 0; 

            // Logique d'affichage
            match tile_id {
                // --- Tuiles Composites (Eau + Sable/Terre) ---
                8 => spawn_composite_tile(&mut commands, &assets, pos, get_atlas_index(0, 0), get_atlas_index(5, 0), Quat::IDENTITY),
                9 => spawn_composite_tile(&mut commands, &assets, pos, get_atlas_index(0, 0), get_atlas_index(5, 0), Quat::from_rotation_z(-90.0f32.to_radians())),
                10 => spawn_composite_tile(&mut commands, &assets, pos, get_atlas_index(0, 0), get_atlas_index(5, 0), Quat::from_rotation_z(180.0f32.to_radians())),
                11 => spawn_composite_tile(&mut commands, &assets, pos, get_atlas_index(0, 0), get_atlas_index(5, 0), Quat::from_rotation_z(90.0f32.to_radians())),
                
                12 => spawn_composite_tile(&mut commands, &assets, pos, get_atlas_index(0, 0), get_atlas_index(6, 0), Quat::IDENTITY),
                13 => spawn_composite_tile(&mut commands, &assets, pos, get_atlas_index(0, 0), get_atlas_index(6, 0), Quat::from_rotation_z(-90.0f32.to_radians())),
                14 => spawn_composite_tile(&mut commands, &assets, pos, get_atlas_index(0, 0), get_atlas_index(6, 0), Quat::from_rotation_z(180.0f32.to_radians())),
                15 => spawn_composite_tile(&mut commands, &assets, pos, get_atlas_index(0, 0), get_atlas_index(6, 0), Quat::from_rotation_z(90.0f32.to_radians())),

                16 => spawn_composite_tile(&mut commands, &assets, pos, get_atlas_index(0, 0), get_atlas_index(4, 0), Quat::IDENTITY),
                17 => spawn_composite_tile(&mut commands, &assets, pos, get_atlas_index(0, 0), get_atlas_index(4, 0), Quat::from_rotation_z(-90.0f32.to_radians())),
                18 => spawn_composite_tile(&mut commands, &assets, pos, get_atlas_index(0, 0), get_atlas_index(4, 0), Quat::from_rotation_z(180.0f32.to_radians())),
                19 => spawn_composite_tile(&mut commands, &assets, pos, get_atlas_index(0, 0), get_atlas_index(4, 0), Quat::from_rotation_z(90.0f32.to_radians())),

                // --- Tuiles Simples ---
                _ => {
                    (index, rotation) = match tile_id {
                        0 => (get_atlas_index(9, 0), Quat::IDENTITY),
                        1 => (get_atlas_index(0, 0), Quat::IDENTITY),
                        2 => (get_atlas_index(8, 0), Quat::IDENTITY),
                        3 => (get_atlas_index(8, 0), Quat::from_rotation_z(-90.0f32.to_radians())),
                        4 => (get_atlas_index(7, 0), Quat::IDENTITY),
                        5 => (get_atlas_index(7, 0), Quat::from_rotation_z(-90.0f32.to_radians())),
                        6 => (get_atlas_index(7, 0), Quat::from_rotation_z(180.0f32.to_radians())),
                        7 => (get_atlas_index(7, 0), Quat::from_rotation_z(90.0f32.to_radians())),

                        20 => { // START
                            spawn_composite_tile(&mut commands, &assets, pos, get_atlas_index(8, 0), get_atlas_index(7, 2), Quat::IDENTITY);
                            (999, Quat::IDENTITY) // 999 = Ne pas spawner de tuile simple supplémentaire
                        },
                        21 => { // END
                            spawn_composite_tile(&mut commands, &assets, pos, get_atlas_index(8, 0), get_atlas_index(8, 2), Quat::IDENTITY);
                            (999, Quat::IDENTITY)
                        },
                        _ => (get_atlas_index(0, 0), Quat::IDENTITY),
                    };

                    if index != 999 {
                        commands.spawn((
                            SpriteSheetBundle {
                                texture: assets.sprite_atlas.clone(),
                                atlas: TextureAtlas {
                                    layout: assets.sprite_atlas_layout.clone(),
                                    index,
                                },
                                transform: Transform {
                                    translation: pos.extend(0.0),
                                    rotation,
                                    ..default()
                                },
                                ..default()
                            },
                            GameTile { tile_type },
                            GameComponent,
                            Name::new(format!("Tile ({x},{y})")),
                        ));
                    }
                }
            }

            // On capture la position de départ pour le pathfinding
            if tile_type == TileType::Start {
                start_pos = pos;
            }
        }
    }

    // construction de la liste des points que l'ennemi devra suivre
    
    let mut path_points = Vec::new();
    path_points.push(start_pos);

    // convertion de la position de départ (pixels) en coordonnées de grille (0-19)
    let mut grid_x = ((start_pos.x - x_offset) / TILE_SIZE).round() as i32;
    let mut grid_y = ((y_offset - start_pos.y) / TILE_SIZE).round() as i32;
    
    let mut last_grid_pos = (grid_x, grid_y); // Pour ne pas revenir en arrière

    // Boucle de recherche de chemin
    for _ in 0..100 { // Sécurité anti-boucle infinie
        let mut found_next = false;
        let neighbors = [(0, -1), (0, 1), (-1, 0), (1, 0)]; // Haut, Bas, Gauche, Droite

        for (dx, dy) in neighbors {
            let nx = grid_x + dx;
            let ny = grid_y + dy;

            // Vérifications limites + pour éviter de revenir en arrière
            if nx < 0 || ny < 0 || nx >= 20 || ny >= 20 { continue; }
            if (nx, ny) == last_grid_pos { continue; }

            let tile_id = level_data[ny as usize][nx as usize];
            let tile_type = get_tile_type(tile_id);

            // Si c'est une route ou la fin, c'est notre prochain point
            if tile_type == TileType::Road || tile_type == TileType::End {
                last_grid_pos = (grid_x, grid_y);
                grid_x = nx;
                grid_y = ny;

                let next_world_pos = Vec2::new(
                    x_offset + nx as f32 * TILE_SIZE,
                    y_offset - ny as f32 * TILE_SIZE,
                );
                path_points.push(next_world_pos);
                found_next = true;
                
                if tile_type == TileType::End {
                    break; // la fin à été trouvée, sortie du 'for neighbors'
                }
                break; // le prochain pas trouvé, passage à l'itération suivante
            }
        }

        if !found_next { break; } // Plus de route trouvée ou cul de sac
        
        // Si on est sur la tuile de fin, tout est terminé
        let current_tile_id = level_data[grid_y as usize][grid_x as usize];
        if get_tile_type(current_tile_id) == TileType::End {
            break;
        }
    }

    println!("Chemin calculé avec succès : {} points", path_points.len());

    // Insertion de la Ressource pour que le système d'Ennemis puisse la lire
    commands.insert_resource(Path {
        points: path_points,
    });
}

fn spawn_ui_button<T: Component>(parent: &mut ChildBuilder, text: &str, marker: T) {
    parent.spawn((
        ButtonBundle {
            style: Style { width: Val::Px(80.0), height: Val::Px(30.0), margin: UiRect::right(Val::Px(10.0)), justify_content: JustifyContent::Center, align_items: AlignItems::Center, border: UiRect::all(Val::Px(2.0)), ..default() },
            background_color: Color::rgb(0.33, 0.23, 0.15).into(), border_color: BorderColor(Color::BLACK), ..default()
        }, marker
    )).with_children(|p| { p.spawn(TextBundle::from_section(text, TextStyle { font_size: 16.0, color: Color::rgb(0.9, 0.85, 0.7), ..default() })); });
}

fn setup_game_ui(mut commands: Commands, assets: Res<GameAssets>) {
    let bar_color = Color::rgb_u8(220, 123, 15);
    commands.spawn((NodeBundle {
        style: Style { width: Val::Percent(100.0), height: Val::Px(100.0), position_type: PositionType::Absolute, bottom: Val::Px(0.0), align_items: AlignItems::Center, padding: UiRect::all(Val::Px(10.0)), ..default() },
        background_color: bar_color.into(), ..default() }, GameComponent,
    )).with_children(|parent| {
        spawn_ui_button(parent, "MENU", BtnMenu);
        spawn_ui_button(parent, "PAUSE", BtnPause);
        parent.spawn(NodeBundle { style: Style { width: Val::Px(20.0), ..default() }, ..default() });
        spawn_tower_button(parent, &assets, TowerType::Canon);
        spawn_tower_button(parent, &assets, TowerType::Archer);
        spawn_tower_button(parent, &assets, TowerType::Wizard);
        parent.spawn((TextBundle::from_section("Gold: 100\nLives: 3", TextStyle { font_size: 20.0, color: Color::BLACK, ..default() }).with_style(Style { margin: UiRect::left(Val::Px(20.0)), ..default() }), MoneyText));
    });

    // Overlay Pause
    commands.spawn((NodeBundle {
        style: Style { width: Val::Percent(100.0), height: Val::Percent(100.0), position_type: PositionType::Absolute, justify_content: JustifyContent::Center, align_items: AlignItems::Center, display: Display::None, ..default() },
        background_color: Color::rgba(0.0, 0.0, 0.0, 0.5).into(), z_index: ZIndex::Global(100), ..default() }, PauseOverlay, GameComponent,
    )).with_children(|parent| {
        parent.spawn(TextBundle::from_section("GAME PAUSED", TextStyle { font_size: 50.0, color: Color::WHITE, ..default() }));
    });
}

fn spawn_tower_button(parent: &mut ChildBuilder, assets: &Res<GameAssets>, tower_type: TowerType) {
    parent.spawn((
        ButtonBundle {
            style: Style {
                width: Val::Px(50.0), // Taille ajustée comme en Java
                height: Val::Px(50.0),
                margin: UiRect::all(Val::Px(10.0)), // Espacement
                justify_content: JustifyContent::Center,
                align_items: AlignItems::Center,
                border: UiRect::all(Val::Px(2.0)),
                ..default()
            },
            // Couleur de fond neutre (Java utilise GRAY)
            background_color: Color::GRAY.into(),
            // Couleur de bordure par défaut (Noir)
            border_color: BorderColor(Color::BLACK), 
            ..default()
        },
        TowerButton { tower_type },
    )).with_children(|parent| {
        // utilisation d'AtlasImageBundle pour afficher JUSTE le sprite
        parent.spawn(AtlasImageBundle {
            style: Style {
                width: Val::Percent(100.0),  // Remplissage du bouton
                height: Val::Percent(100.0),
                ..default()
            },
            texture_atlas: TextureAtlas {
                layout: assets.sprite_atlas_layout.clone(),
                index: tower_type.get_sprite_index(),
            },
            image: UiImage::new(assets.sprite_atlas.clone()),
            ..default()
        });
    });
}

pub fn spawn_composite_tile(commands: &mut Commands, assets: &Res<GameAssets>, pos: Vec2, base_i: usize, over_i: usize, rot: Quat) {
     commands.spawn((
        SpriteSheetBundle {
            texture: assets.sprite_atlas.clone(),
            atlas: TextureAtlas { layout: assets.sprite_atlas_layout.clone(), index: base_i },
            transform: Transform::from_xyz(pos.x, pos.y, 0.0), ..default()
        }, GameTile { tile_type: TileType::Water }, GameComponent));
    commands.spawn((
        SpriteSheetBundle {
            texture: assets.sprite_atlas.clone(),
            atlas: TextureAtlas { layout: assets.sprite_atlas_layout.clone(), index: over_i },
            transform: Transform { translation: pos.extend(0.1), rotation: rot, ..default() }, ..default()
        }, GameTile { tile_type: TileType::Water }, GameComponent));
}

// Gère le clic sur les boutons du bas
fn tower_button_interaction(
    mut interaction_query: Query<
        (&Interaction, &TowerButton, &mut BorderColor),
        With<Button>,
    >,
    mut selected_tower: ResMut<SelectedTower>,
) {
    for (interaction, tower_button, mut border_color) in interaction_query.iter_mut() {
        
        // le bouton correspond-il à la tour sélectionnée ?
        let is_selected = selected_tower.tower_type == Some(tower_button.tower_type);

        match *interaction {
            Interaction::Pressed => {
                selected_tower.tower_type = Some(tower_button.tower_type);
                *border_color = BorderColor(Color::BLACK);
            }
            Interaction::Hovered => {
                // Hover : Bordure blanche
                *border_color = BorderColor(Color::WHITE);
            }
            Interaction::None => {
                // Normal
                if is_selected {
                    *border_color = BorderColor(Color::BLACK); 
                } else {
                    *border_color = BorderColor(Color::GRAY);
                }
            }
        }
    }
}

fn update_ui_stats(
    stats: Res<PlayerStats>,
    mut money_query: Query<&mut Text, (With<MoneyText>, Without<LivesText>)>,
    mut lives_query: Query<&mut Text, (With<LivesText>, Without<MoneyText>)>,
) {
    // mise a jour du texte
    for mut text in money_query.iter_mut() {
        text.sections[0].value = format!("Gold: {}", stats.money);
    }
    for mut text in lives_query.iter_mut() {
        text.sections[0].value = format!("Lives: {}", stats.lives);
    }
}

fn ui_button_interaction(
    mut interaction_query: Query<(&Interaction, Option<&BtnMenu>, Option<&BtnPause>), (Changed<Interaction>, With<Button>)>,
    mut next_state: ResMut<NextState<AppState>>,
    mut pause: ResMut<GlobalPause>,
) {
    for (interaction, btn_menu, btn_pause) in interaction_query.iter_mut() {
        if *interaction == Interaction::Pressed {
            if btn_menu.is_some() {
                next_state.set(AppState::Menu);
                pause.0 = false; 
            } else if btn_pause.is_some() {
                pause.0 = !pause.0; // Toggle
            }
        }
    }
}

fn update_ui_text(
    stats: Res<PlayerStats>,
    pause: Res<GlobalPause>,
    mut money_query: Query<&mut Text, With<MoneyText>>,
    mut pause_btn_text: Query<&mut Text, (With<BtnPause>, Without<MoneyText>)>,
    mut overlay_query: Query<&mut Style, With<PauseOverlay>>,
) {
    for mut text in money_query.iter_mut() { text.sections[0].value = format!("Gold: {}\nLives: {}", stats.money, stats.lives); }
    for mut text in pause_btn_text.iter_mut() { text.sections[0].value = if pause.0 { "RESUME".into() } else { "PAUSE".into() }; }
    for mut style in overlay_query.iter_mut() { style.display = if pause.0 { Display::Flex } else { Display::None }; }
}

// Gère le clic sur la grille pour poser une tour
fn grid_click_interaction(mut commands: Commands, mouse: Res<ButtonInput<MouseButton>>, win: Query<&Window, With<PrimaryWindow>>, cam: Query<(&Camera, &GlobalTransform)>, sel: Res<SelectedTower>, ass: Res<GameAssets>, mut stats: ResMut<PlayerStats>, pause: Res<GlobalPause>) {
    if pause.0 || !mouse.just_pressed(MouseButton::Left) { return; } 
    let Some(tt) = sel.tower_type else { return; };
    let cost = tt.get_cost();
    if stats.money < cost { return; }
    let (cam, c_trans) = cam.single();
    let Some(w) = win.get_single().ok() else { return; };
    if let Some(w_pos) = w.cursor_position().and_then(|c| cam.viewport_to_world(c_trans, c)).map(|r| r.origin.truncate()) {
        if w_pos.y < (-370.0 + 100.0) { return; } 
        let ts = 32.0;
        let x_off = -20.0 * ts / 2.0 + ts/2.0;
        let y_off = (20.0 * ts / 2.0 - ts/2.0) + 50.0;
        let gx = ((w_pos.x - x_off)/ts).round();
        let gy = ((y_off - w_pos.y)/ts).round();
        if gx>=0.0 && gx<20.0 && gy>=0.0 && gy<20.0 {
             let lvl = level::get_level_data();
             if lvl[gy as usize][gx as usize] == 0 {
                 let snap = Vec2::new(x_off + gx*ts, y_off - gy*ts);
                 let (rng, dmg, cd) = tt.get_base_stats();
                 commands.spawn((
                     SpriteSheetBundle {
                         texture: ass.sprite_atlas.clone(),
                         atlas: TextureAtlas { layout: ass.sprite_atlas_layout.clone(), index: tt.get_sprite_index() },
                         transform: Transform::from_xyz(snap.x, snap.y, 2.0), ..default()
                     }, Tower { range: rng, damage: dmg, cooldown: Timer::from_seconds(cd, TimerMode::Repeating) }, GameComponent
                 ));
                 stats.money -= cost;
             }
        }
    }
}

// Traduit la logique de TileManager.java
// et Constants.java
pub fn get_tile_type(tile_id: u32) -> TileType {
    match tile_id {
        // ID 0 = GRASS_TILE (type 1)
        0 => TileType::Grass,
        
        // IDs 2-7 = ROAD_TILE (type 2)
        2..=7 => TileType::Road,
        
        // ID 20 = START_TILE (type 3)
        20 => TileType::Start,
        
        // ID 21 = END_TILE (type 4)
        21 => TileType::End,
        
        // Tous les autres (1, 8-19) sont WATER_TILE (type 0)
        _ => TileType::Water,
    }
}

fn cleanup_game(
    mut cmd: Commands, 
    // Récupération de toutes les entités du jeu
    all_enemies: Query<Entity, With<Enemy>>,
    all_towers: Query<Entity, With<Tower>>,
    all_projectiles: Query<Entity, With<Projectile>>,
    all_tiles: Query<Entity, With<GameTile>>,
    all_ui: Query<Entity, With<GameComponent>>
) {
    println!("Nettoyage TOTAL du jeu...");
    for e in all_enemies.iter() { cmd.entity(e).despawn_recursive(); }
    for e in all_towers.iter() { cmd.entity(e).despawn_recursive(); }
    for e in all_projectiles.iter() { cmd.entity(e).despawn_recursive(); }
    for e in all_tiles.iter() { cmd.entity(e).despawn_recursive(); }
    for e in all_ui.iter() { cmd.entity(e).despawn_recursive(); }
    
    cmd.remove_resource::<Path>();
    cmd.remove_resource::<SelectedTower>();
    cmd.remove_resource::<PlayerStats>();
}

pub fn tower_shooting(mut commands: Commands, assets: Res<GameAssets>, time: Res<Time>, mut tower_query: Query<(&Transform, &mut Tower)>, enemy_query: Query<(Entity, &Transform), With<Enemy>>) {
    for (t_trans, mut tower) in tower_query.iter_mut() {
        tower.cooldown.tick(time.delta());
        if tower.cooldown.just_finished() {
            let t_pos = t_trans.translation.truncate();
            let mut closest = None;
            let mut min_sq = tower.range * tower.range;
            for (e_ent, e_trans) in enemy_query.iter() {
                let d_sq = t_pos.distance_squared(e_trans.translation.truncate());
                if d_sq <= min_sq { min_sq = d_sq; closest = Some(e_ent); }
            }
            if let Some(target) = closest {
                commands.spawn((
                    SpriteSheetBundle {
                        texture: assets.sprite_atlas.clone(),
                        atlas: TextureAtlas { layout: assets.sprite_atlas_layout.clone(), index: 17 },
                        transform: Transform::from_xyz(t_pos.x, t_pos.y, 2.0), ..default()
                    }, Projectile { target, damage: tower.damage, speed: 300.0 }, GameComponent
                ));
                tower.cooldown.reset();
            }
        }
    }
}