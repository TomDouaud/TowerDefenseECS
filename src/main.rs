use bevy::{prelude::*, app::AppExit, asset::AssetServer, diagnostic::{FrameTimeDiagnosticsPlugin, LogDiagnosticsPlugin},};

// enum de tous les états possibles de l'application (basé sur GameState.java)
#[derive(States, Debug, Clone, Eq, PartialEq, Hash, Default)]
pub enum AppState {
    #[default] // L'état par défaut au lancement
    Menu,
    Playing,
    Simulation,
}


pub mod menu;
pub mod game;
pub mod constants;
pub mod level;
pub mod enemy;
pub mod tower;
pub mod projectile;
pub mod simulation;

use menu::MenuPlugin;
use game::GamePlugin;
use enemy::EnemyPlugin;
use simulation::SimulationPlugin;
use projectile::ProjectilePlugin;

// définition des ressources de jeu (images, atlas, etc)
#[derive(Resource)]
pub struct GameAssets {
    pub menu_background: Handle<Image>,
    pub sprite_atlas: Handle<Image>,
    pub sprite_atlas_layout: Handle<TextureAtlasLayout>,
}

#[derive(Resource, Default)]
pub struct GlobalPause(pub bool);

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "Tower Defense (Bevy)".into(),
                    resolution: (640.0, 740.0).into(), 
                    resizable: false,
                    ..default()
                }),
                ..default()
            }),
            FrameTimeDiagnosticsPlugin::default(),
        ))
        .init_state::<AppState>()
        .init_resource::<GlobalPause>() 
        .add_plugins((
            MenuPlugin,
            GamePlugin,
            // SettingsPlugin supprimé
            EnemyPlugin,
            ProjectilePlugin,
            SimulationPlugin,
        ))
        .add_systems(Startup, (setup_camera, setup_assets))
        .run();
}

/// Lancement de la caméra 2D dans le monde
fn setup_camera(mut commands: Commands) {
    commands.spawn(Camera2dBundle::default());
}

// Système de chargement des assets et création de la ressource GameAssets au démarrage
fn setup_assets(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    // chargement des images
    let menu_bg_handle = asset_server.load("menuimg.png");
    let atlas_img_handle = asset_server.load("spriteatlas.png");

    // définition du layout qui est le découpage de l'atlas
    // l'atlas fait 10 sprites de large par 3 de haut
    let atlas_layout = TextureAtlasLayout::from_grid(
        Vec2::new(32.0, 32.0), // Taille de chaque sprite
        10,                    // Nombre de colonnes
        3,                     // Nombre de lignes
        None,                  // Pas de padding
        None,                  // Pas d'offset
    );
    let atlas_layout_handle = texture_atlas_layouts.add(atlas_layout);

    // Création de la ressource GameAssets
    let game_assets = GameAssets {
        menu_background: menu_bg_handle,
        sprite_atlas: atlas_img_handle,
        sprite_atlas_layout: atlas_layout_handle,
    };

    // Insertion dans le monde
    commands.insert_resource(game_assets);

    println!("Assets chargés et Ressource 'GameAssets' créée !");
}