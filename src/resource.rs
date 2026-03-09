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
            output_qty: 4.0,
            inputs: &[],
        },
        // Lumberjack: produces timber from nothing (primary resource)
        Recipe {
            output: Resource::Timber,
            output_qty: 4.0,
            inputs: &[],
        },
        // Miner: produces iron ore from nothing (primary resource)
        Recipe {
            output: Resource::IronOre,
            output_qty: 2.0,
            inputs: &[],
        },
        // Miller: grain -> flour (processing adds value, more output than input)
        Recipe {
            output: Resource::Flour,
            output_qty: 5.0,
            inputs: &[(Resource::Grain, 2.0)],
        },
        // Sawmill: timber -> planks (processing adds value)
        Recipe {
            output: Resource::Planks,
            output_qty: 4.0,
            inputs: &[(Resource::Timber, 2.0)],
        },
        // Smelter: iron ore + timber -> iron ingots
        Recipe {
            output: Resource::IronIngots,
            output_qty: 2.0,
            inputs: &[(Resource::IronOre, 2.0), (Resource::Timber, 1.0)],
        },
        // Blacksmith: iron ingots + planks -> tools
        Recipe {
            output: Resource::Tools,
            output_qty: 2.0,
            inputs: &[(Resource::IronIngots, 1.0), (Resource::Planks, 2.0)],
        },
    ]
}
