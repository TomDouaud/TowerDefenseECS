use bevy::prelude::*;
use crate::{AppState, enemy::Enemy, enemy::Health, GlobalPause};

// Composant Projectile
#[derive(Component)]
pub struct Projectile {
    pub target: Entity, // L'entité ennemie visée
    pub damage: i32,
    pub speed: f32,
}

pub struct ProjectilePlugin;

fn not_paused(pause: Res<GlobalPause>) -> bool { !pause.0 }

impl Plugin for ProjectilePlugin {
    fn build(&self, app: &mut App) {
        // On exécute si le jeu n'est pas en pause
        app.add_systems(Update, move_projectiles.run_if(not_paused));
    }
}

fn move_projectiles(
    mut commands: Commands,
    mut projectile_query: Query<(Entity, &mut Transform, &Projectile)>,
    // On cherche n'importe quelle entité qui a une Transform (Ennemi normal ou Sim)
    target_query: Query<&GlobalTransform>, 
    time: Res<Time>,
    // On cherche n'importe quelle entité qui a de la vie
    mut health_query: Query<&mut Health>,
) {
    for (proj_entity, mut proj_transform, projectile) in projectile_query.iter_mut() {
        
        // Si la cible existe toujours
        if let Ok(target_transform) = target_query.get(projectile.target) {
            
            let target_pos = target_transform.translation().truncate();
            let current_pos = proj_transform.translation.truncate();
            let direction = target_pos - current_pos;
            let distance = direction.length();
            
            let step = projectile.speed * time.delta_seconds();

            if distance <= step {
                // Application des dégâts
                if let Ok(mut health) = health_query.get_mut(projectile.target) {
                    health.current -= projectile.damage;
                }
                
                // Détruire le projectile
                commands.entity(proj_entity).despawn();
            } else {
                // Avancer
                let movement = direction.normalize() * step;
                proj_transform.translation.x += movement.x;
                proj_transform.translation.y += movement.y;
                
                let angle = direction.y.atan2(direction.x);
                proj_transform.rotation = Quat::from_rotation_z(angle);
            }

        } else {
            // Cible disparue/morte
            commands.entity(proj_entity).despawn();
        }
    }
}