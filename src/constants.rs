#![allow(dead_code)] // Pour désactiver les avertissements de code non utilisé

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Direction {
    Left,
    Up,
    Right,
    Down,
}

// Équivalent de la classe Constants.Tiles en Java
pub mod tiles {
    pub const WATER_TILE: u32 = 0;
    pub const GRASS_TILE: u32 = 1;
    pub const ROAD_TILE: u32 = 2;
    pub const START_TILE: u32 = 3;
    pub const END_TILE: u32 = 4;
}

// Équivalent de la classe Constants.Enemies en Java
pub mod enemies {
    pub const ORC: u32 = 0;
    pub const BAT: u32 = 1;
    pub const KNIGHT: u32 = 2;
    pub const WOLF: u32 = 3;

    pub fn get_speed(enemy_type: u32) -> f32 {
        match enemy_type {
            ORC => 0.5,
            BAT => 0.7,
            KNIGHT => 0.45,
            WOLF => 0.85,
            _ => 0.0,
        }
    }

    pub fn get_starthealth(enemy_type: u32) -> u32 {
        match enemy_type {
            ORC => 85,
            BAT => 100,
            KNIGHT => 400,
            WOLF => 125,
            _ => 0,
        }
    }
    
}

// Équivalent de la classe Constants.Towers en Java
pub mod towers {
    pub const CANON_TOWER: u32 = 0;
    pub const ARCHER_TOWER: u32 = 1;
    pub const WIZARD_TOWER: u32 = 2;
    
    pub fn get_name(tower_type: u32) -> &'static str {
        match tower_type {
            CANON_TOWER => "Canon Tower",
            ARCHER_TOWER => "Archer Tower",
            WIZARD_TOWER => "Wizard Tower",
            _ => "Unknown Tower",
        }
    }

    pub fn get_startdamage(tower_type: u32) -> u32 {
        match tower_type {
            CANON_TOWER => 15,
            ARCHER_TOWER => 5,
            WIZARD_TOWER => 30,
            _ => 0,
        }
    }

    pub fn get_defaultrange(tower_type: u32) -> u32 {
        match tower_type {
            CANON_TOWER => 75,
            ARCHER_TOWER => 120,
            WIZARD_TOWER => 100,
            _ => 0,
        }
    }

    pub fn get_cooldowntime(tower_type: u32) -> u32 {
        match tower_type {
            CANON_TOWER => 120,
            ARCHER_TOWER => 35,
            WIZARD_TOWER => 50,
            _ => 0,
        }
    }
}

pub mod projectiles {
    pub const ARROW : u32 = 0;
    pub const BOMB : u32 = 1;
    pub const CHAINS : u32 = 2;

    pub fn get_speed(projectile_type: u32) -> f32 {
        match projectile_type {
            ARROW => 8.0,
            BOMB => 4.0,
            CHAINS => 6.0,
            _ => 0.0,
        }
    }
}