use bevy::prelude::*;

// Les 3 types de tours dans le jeu
#[derive(Debug, Clone, Copy, PartialEq, Eq, Component)]
pub enum TowerType {
    Canon = 0,
    Archer = 1,
    Wizard = 2,
}

// Composant principal d'une tour (sa portée, ses dégâts, son cooldown)
#[derive(Component)]
pub struct Tower {
    pub range: f32,
    pub damage: i32,
    pub cooldown: Timer,
}

impl TowerType {
    // Valeurs tirées de Constants.java
    // Canon: Dmg 15, Range 75, CD 120 ticks (2.0s à 60 UPS) -> Bevy Timer 2.0s
    // Archer: Dmg 5, Range 120, CD 35 ticks (0.58s)
    // Wizard: Dmg 0 (ou 1?), Range 100, CD 50 ticks (0.83s)
    
    pub fn get_base_stats(&self) -> (f32, i32, f32) {
        // (Range, Damage, Cooldown_Sec)
        match self {
            TowerType::Canon => (75.0, 15, 2.0),
            TowerType::Archer => (120.0, 5, 0.583),
            TowerType::Wizard => (100.0, 1, 0.833), 
        }
    }

    // Stats Tier 3 pour la simulation (Upgrade x2)
    // Java: Canon (+10 dmg, +20 range, -30 ticks CD)
    // Java: Archer (+4 dmg, +40 range, -10 ticks CD)
    // Java: Wizard (+2 dmg, +30 range, -20 ticks CD)
    pub fn get_sim_stats(&self) -> (f32, i32, f32) {
        match self {
            TowerType::Canon => (95.0, 25, 1.5),      // 120 - 30 = 90 ticks = 1.5s
            TowerType::Archer => (160.0, 9, 0.416),   // 35 - 10 = 25 ticks = 0.416s
            TowerType::Wizard => (130.0, 3, 0.5),     // 50 - 20 = 30 ticks = 0.5s
        }
    }

    pub fn get_cost(&self) -> i32 {
        match self {
            TowerType::Canon => 65,
            TowerType::Archer => 35,
            TowerType::Wizard => 50,
        }
    }

    pub fn get_sprite_index(&self) -> usize {
        match self {
            TowerType::Canon => 14,
            TowerType::Archer => 15,
            TowerType::Wizard => 16,
        }
    }
}