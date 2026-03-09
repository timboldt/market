use strum_macros::EnumIter;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, EnumIter)]
#[repr(usize)]
pub enum Resource {
    Grain = 0,
    Timber = 1,
    Stone = 2,
    IronOre = 3,
    Wool = 4,
    Clay = 5,
    Herbs = 6,
    Flour = 7,
    Planks = 8,
    IronIngots = 9,
    Tools = 10,
    Cloth = 11,
}

impl Resource {
    pub fn short_name(&self) -> &'static str {
        match self {
            Resource::Grain => "Grain",
            Resource::Timber => "Timbr",
            Resource::Stone => "Stone",
            Resource::IronOre => "IrOre",
            Resource::Wool => "Wool ",
            Resource::Clay => "Clay ",
            Resource::Herbs => "Herbs",
            Resource::Flour => "Flour",
            Resource::Planks => "Plank",
            Resource::IronIngots => "Ingot",
            Resource::Tools => "Tools",
            Resource::Cloth => "Cloth",
        }
    }
}

impl Resource {
    pub fn spoilage_rate(&self) -> f32 {
        use crate::config::*;
        match self {
            Resource::Grain | Resource::Flour | Resource::Herbs => SPOILAGE_PERISHABLE,
            Resource::Timber
            | Resource::Wool
            | Resource::Clay
            | Resource::IronOre
            | Resource::Stone => SPOILAGE_RAW,
            Resource::Planks | Resource::IronIngots | Resource::Tools | Resource::Cloth => {
                SPOILAGE_PROCESSED
            }
        }
    }
}

impl std::fmt::Display for Resource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

pub struct Recipe {
    pub output: Resource,
    pub output_qty: f32,
    pub inputs: &'static [(Resource, f32)],
}

pub fn all_recipes() -> Vec<Recipe> {
    vec![
        // Farmer: produces grain from nothing (primary resource)
        Recipe {
            output: Resource::Grain,
            output_qty: 2.5,
            inputs: &[],
        },
        // Lumberjack: produces timber from nothing (primary resource)
        Recipe {
            output: Resource::Timber,
            output_qty: 1.5,
            inputs: &[],
        },
        // Miner: produces iron ore from nothing (primary resource)
        Recipe {
            output: Resource::IronOre,
            output_qty: 1.0,
            inputs: &[],
        },
        // Miller: grain -> flour (processing adds value, more output than input)
        Recipe {
            output: Resource::Flour,
            output_qty: 3.0,
            inputs: &[(Resource::Grain, 1.0)],
        },
        // Sawmill: timber -> planks (processing adds value)
        Recipe {
            output: Resource::Planks,
            output_qty: 2.0,
            inputs: &[(Resource::Timber, 1.0)],
        },
        // Smelter: iron ore + timber -> iron ingots
        Recipe {
            output: Resource::IronIngots,
            output_qty: 1.5,
            inputs: &[(Resource::IronOre, 1.0), (Resource::Timber, 0.5)],
        },
        // Blacksmith: iron ingots + planks -> tools
        Recipe {
            output: Resource::Tools,
            output_qty: 3.0,
            inputs: &[(Resource::IronIngots, 1.0), (Resource::Planks, 1.0)],
        },
        // Shepherd: produces wool from nothing (primary resource)
        Recipe {
            output: Resource::Wool,
            output_qty: 1.5,
            inputs: &[],
        },
        // Weaver: wool -> cloth (processing adds value)
        Recipe {
            output: Resource::Cloth,
            output_qty: 2.0,
            inputs: &[(Resource::Wool, 1.0)],
        },
    ]
}
