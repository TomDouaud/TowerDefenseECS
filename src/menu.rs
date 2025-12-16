use bevy::{
    prelude::*, 
    app::AppExit};
use crate::AppState; 
use crate::GameAssets;

// Composant vide pour marquer les entitées créées par le menu pour les trouver et les supprimer facilement si besoin
#[derive(Component)]
struct MenuUI;

// Composant pour les boutons, avec les actions associées
#[derive(Component)]
enum MenuButtonAction {
    Play,
    Simulation,
    Quit,
}

// Couleurs des boutons
const NORMAL_BUTTON: Color = Color::rgb(0.15, 0.15, 0.15);
const HOVERED_BUTTON: Color = Color::rgb(0.25, 0.25, 0.25);
const PRESSED_BUTTON: Color = Color::rgb(0.35, 0.75, 0.35);

// équivalent de la classe "Menu.java"
pub struct MenuPlugin;

impl Plugin for MenuPlugin {
    fn build(&self, app: &mut App) {
        app
            // Ajouts des systemes qui s'éxécutent lors des changements d'état
            
            // S'exécute 1x quand on *entre* dans AppState::Menu
            .add_systems(OnEnter(AppState::Menu), setup_menu)
            
            // S'exécute à chaque frame quand on est dans AppState::Menu
            .add_systems(Update, 
                (button_interaction_system)
                .run_if(in_state(AppState::Menu))
            )

            // S'exécute 1x quand on *sort* de AppState::Menu
            .add_systems(OnExit(AppState::Menu), cleanup_menu);
    }
}

// équivalent du constructeur Menu() ou initButtons() en java
fn setup_menu(mut commands: Commands, assets: Res<GameAssets>) {
    println!("Bienvenue au Menu !");
    
    // Image du fond
    commands.spawn((
        ImageBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                ..default()
            },
            image: assets.menu_background.clone().into(),
            ..default()
        },
        MenuUI, // marque l'image pour le cleanup
    ));

    // Spawn le conteneur pour les boutons
    // NodeBundle est la pour centrer nos boutons verticalement
    commands.spawn((
        ImageBundle {
            style: Style {
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                flex_direction: FlexDirection::Column, // Aligne les enfants verticalement
                align_items: AlignItems::Center,     // Centre horizontalement
                justify_content: JustifyContent::Center, // Centre verticalement
                ..default()
            },
            background_color: Color::NONE.into(), // Transparent
            ..default()
        },
        MenuUI, // marque le conteneur pour le cleanup
    ))
    .with_children(|parent| {
        // Dimensions des boutons de Menu.java
        let button_style = Style {
            width: Val::Px(150.0),
            height: Val::Px(50.0), // 150 / 3
            margin: UiRect::all(Val::Px(10.0)), // Espace entre les boutons
            justify_content: JustifyContent::Center, // Centre le texte
            align_items: AlignItems::Center,     // Centre le texte
            ..default()
        };
        
        let text_style = TextStyle {
            font_size: 24.0,
            color: Color::WHITE,
            ..default() // police par défaut de Bevy
        };

        // --- Bouton "PLAY" ---
        parent.spawn((
            ButtonBundle {
                style: button_style.clone(),
                background_color: NORMAL_BUTTON.into(),
                ..default()
            },
            MenuButtonAction::Play, // Ajoute le composant d'action
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section("PLAY", text_style.clone()));
        });
        // --- Bouton "SIMULATION" ---
        parent.spawn((
        ButtonBundle {
            style: button_style.clone(),
            background_color: NORMAL_BUTTON.into(),
            ..default()
            },
            MenuButtonAction::Simulation,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section("SIMULATION", text_style.clone()));
        });
        // --- Bouton "QUIT" ---
        parent.spawn((
            ButtonBundle {
                style: button_style.clone(),
                background_color: NORMAL_BUTTON.into(),
                ..default()
            },
            MenuButtonAction::Quit,
        ))
        .with_children(|parent| {
            parent.spawn(TextBundle::from_section("QUIT", text_style.clone()));
        });
    });
}

// nettoyage du menu quand on en sort
fn cleanup_menu(mut commands: Commands, query: Query<Entity, With<MenuUI>>) {
    println!("Nettoyage du Menu...");
    for entity in query.iter() {
        // Supprime (despawn) toutes les entités marquées avec "MenuUI"
        commands.entity(entity).despawn_recursive();
    }
}

// systeme d'interaction avec les boutons
fn button_interaction_system(
    mut interaction_query: Query<
        (Entity, &Interaction, &MenuButtonAction), // récupération de l'action
        (Changed<Interaction>, With<Button>),
    >,
    mut next_state: ResMut<NextState<AppState>>, // Pour changer l'état
    mut app_exit_writer: EventWriter<AppExit>, // Pour quitter le jeu
    mut button_query: Query<&mut BackgroundColor, With<Button>>, // Pour changer la couleur
) {
    for (entity, interaction, action) in interaction_query.iter_mut() {
        
        let mut background_color = button_query.get_mut(entity).unwrap();

        match *interaction {
            Interaction::Pressed => {
                *background_color = PRESSED_BUTTON.into();
                // Exécute l'action associée au bouton
                match action {
                    MenuButtonAction::Play => {
                        println!("Bouton Play cliqué !");
                        next_state.set(AppState::Playing); // Change l'état
                    }
                    MenuButtonAction::Simulation => {
                        println!("Bouton Simulation cliqué !");
                        next_state.set(AppState::Simulation); // Change l'état
                    }
                    MenuButtonAction::Quit => {
                        println!("Bouton Quit cliqué !");
                        app_exit_writer.send(AppExit); // Envoie l'événement pour quitter
                    }
                }
            }
            Interaction::Hovered => {
                *background_color = HOVERED_BUTTON.into();
            }
            Interaction::None => {
                *background_color = NORMAL_BUTTON.into();
            }
        }
    }
}