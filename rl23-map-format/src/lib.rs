use std::{
    fs::{File},
    io::{Read},
    path::PathBuf
};
use std::collections::HashMap;
use rand::Rng;
use ron::{
    de::from_reader,
    ser::{PrettyConfig, to_writer_pretty}
};
use serde::{
    Deserialize,
    Serialize
};

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MapInfo {
    pub width: usize,
    pub height: usize,
    pub terrain_layer: Vec<TerrainKind>,
    #[serde(default)]
    pub entity_layer: HashMap<usize, MapEntity>,
    pub wall_layer: Vec<Option<WallKind>>,
}
impl MapInfo {
    pub fn create_new(width: usize, height: usize) -> Self {
        let size = width * height;
        let mut rng = rand::thread_rng();
        let wall_layer = vec![None; size];
        let mut terrain_layer = Vec::with_capacity(size);
        for _ in 0..size {
            terrain_layer.push(TerrainKind::Mud { offset: rng.gen_range(0..12) })
        }
        Self {
            width,
            height,
            terrain_layer,
            entity_layer: Default::default(),
            wall_layer
        }
    }

    pub fn read_from_path(path: &PathBuf) -> Self {
        let mut bytes = Vec::new();
        let mut file = File::open(path).unwrap();
        file.read_to_end(&mut bytes).unwrap();
        from_reader(&bytes[..]).unwrap()
    }

    pub fn save_to_path(&self, path: &PathBuf) {
        let mut file = File::create(path).unwrap();
        to_writer_pretty(&mut file, self, PrettyConfig::new()).unwrap();
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Deserialize, Serialize)]
pub enum WallKind {
    Dirt,
    Bricks
}
impl WallKind {
    pub fn get_tiling_info(self) -> TilingInfo {
        match self {
            WallKind::Dirt => TilingInfo::Wang(WangTerrain{
                x_offset: 12 * 32,
                y_offset: 0
            }),
            WallKind::Bricks => TilingInfo::Wang(WangTerrain{
                x_offset: 12 * 32,
                y_offset: 4 * 32
            }),
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Deserialize, Serialize)]
pub enum TerrainKind {
    Mud { offset: usize },
    Sand,
    Dirt,
    Grass,
    Water,
    CaveWater,
    Lava,
    Tile,
    BrightTile,
    MossTile,
    VibrantTile
}

impl TerrainKind {
    pub fn get_tiling_info(self) -> TilingInfo {
        match self {
            TerrainKind::Mud { offset } => TilingInfo::Mud(MudTerrain { offset }),
            TerrainKind::Sand => TilingInfo::Wang(WangTerrain{
                x_offset: 4 * 32,
                y_offset: 0
            }),
            TerrainKind::Dirt => TilingInfo::Wang(WangTerrain{
                x_offset: 4 * 32,
                y_offset: 4 * 32
            }),
            TerrainKind::Grass => TilingInfo::Wang(WangTerrain{
                x_offset: 8 * 32,
                y_offset: 4 * 32
            }),
            TerrainKind::Water => TilingInfo::Wang(WangTerrain{
                x_offset: 8 * 32,
                y_offset: 0
            }),
            TerrainKind::CaveWater => TilingInfo::Wang(WangTerrain{
                x_offset: 8 * 32,
                y_offset: 8 * 32
            }),
            TerrainKind::Lava => TilingInfo::Wang(WangTerrain{
                x_offset: 4 * 32,
                y_offset: 8 * 32
            }),
            TerrainKind::Tile => TilingInfo::Wang(WangTerrain{
                x_offset: 12 * 32,
                y_offset: 8 * 32
            }),
            TerrainKind::BrightTile => TilingInfo::Wang(WangTerrain{
                x_offset: 4 * 32,
                y_offset: 12 * 32
            }),
            TerrainKind::MossTile =>  TilingInfo::Wang(WangTerrain{
                x_offset: 8 * 32,
                y_offset: 12 * 32
            }),
            TerrainKind::VibrantTile => TilingInfo::Wang(WangTerrain{
                x_offset: 12 * 32,
                y_offset: 12 * 32
            }),
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub enum TilingInfo {
    Wang(WangTerrain),
    Mud(MudTerrain)
}

#[derive(Copy, Clone, Debug)]
pub struct WangTerrain{ pub x_offset: usize, pub y_offset: usize }
impl WangTerrain{
    pub fn get_final_coords(self, encoding: WangEncoding) -> Option<[usize; 2]> {
        let Self{ x_offset, y_offset} = self;
        match encoding {
            WangEncoding {
                north_east: true,
                north_west: false,
                south_east: false,
                south_west: false
            } => Some([x_offset + 64, y_offset + 32]),
            WangEncoding {
                north_east: false,
                north_west: true,
                south_east: false,
                south_west: false
            } => Some([x_offset + 96, y_offset + 32]),
            WangEncoding {
                north_east: false,
                north_west: false,
                south_east: true,
                south_west: false
            } => Some([x_offset + 64, y_offset]),
            WangEncoding {
                north_east: false,
                north_west: false,
                south_east: false,
                south_west: true
            } => Some([x_offset + 96, y_offset]),
            WangEncoding {
                north_east: true,
                north_west: true,
                south_east: false,
                south_west: false
            } => Some([x_offset, y_offset]),
            WangEncoding {
                north_east: true,
                north_west: false,
                south_east: true,
                south_west: false
            } => Some([x_offset + 32, y_offset + 32]),
            WangEncoding {
                north_east: false,
                north_west: true,
                south_east: false,
                south_west: true
            } => Some([x_offset + 32, y_offset + 96]),
            WangEncoding {
                north_east: false,
                north_west: false,
                south_east: true,
                south_west: true
            } => Some([x_offset, y_offset + 64]),
            WangEncoding {
                north_east: true,
                north_west: false,
                south_east: true,
                south_west: true
            } => Some([x_offset + 96, y_offset + 96]),
            WangEncoding {
                north_east: false,
                north_west: true,
                south_east: true,
                south_west: true
            } => Some([x_offset + 64, y_offset + 96]),
            WangEncoding {
                north_east: true,
                north_west: true,
                south_east: true,
                south_west: false
            } => Some([x_offset + 96, y_offset + 64]),
            WangEncoding {
                north_east: true,
                north_west: true,
                south_east: false,
                south_west: true
            } => Some([x_offset + 64, y_offset + 64]),
            WangEncoding {
                north_east: true,
                north_west: false,
                south_east: false,
                south_west: true
            } => Some([x_offset + 32, y_offset + 64]),
            WangEncoding {
                north_east: false,
                north_west: true,
                south_east: true,
                south_west: false
            } => Some([x_offset + 32, y_offset]),
            WangEncoding {
                north_east: true,
                north_west: true,
                south_east: true,
                south_west: true
            } => Some([x_offset, y_offset + 96]),
            _ => None
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct MudTerrain { offset: usize }
impl MudTerrain {
    pub fn get_variant_coords_by_offset(self) -> [usize; 2] {
        match self.offset % 12 {
            0 => [4 * 32, 32],
            1 => [8 * 32, 32],
            2 => [12 * 32, 32],

            3 => [4 * 32, 5 * 32],
            4 => [8 * 32, 5 * 32],
            5 => [12 * 32, 5 * 32],

            6 => [4 * 32, 9 * 32],
            7 => [8 * 32, 9 * 32],
            8 => [12 * 32, 9 * 32],

            9 => [4 * 32, 13 * 32],
            10 => [8 * 32, 13 * 32],
            11 => [12 * 32, 13 * 32],

            _ => unreachable!()
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct WangEncoding {
    pub north_east: bool,
    pub north_west: bool,
    pub south_east: bool,
    pub south_west: bool
}

#[derive(Copy, Clone, Debug, PartialEq, Deserialize, Serialize)]
pub enum MapEntity {
    Door,
    ClosedDoor(ClosedDoor),
    Unit(Unit)
}

impl MapEntity {
    pub fn get_coords(self) -> [usize; 2] {
        match self {
            MapEntity::Door => [64, 352],
            MapEntity::ClosedDoor(closed_door) => closed_door.get_coords(),
            MapEntity::Unit(unit) => unit.get_coords()
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Deserialize, Serialize)]
pub enum Unit {
    Fighter,
    Archer,
    WhiteMage,
    RedMage,
    OrcSword,
    OrcAxe,
    GoblinFighter,
    GoblinArcher,
    Squirrel,
    Spider,
    Bat,
    Ghost,
    Skeleton1,
    Skeleton2,
    Necromancer
}

impl Unit {
    pub fn get_coords(self) -> [usize; 2] {
        match self {
            Unit::Fighter => [0, 0],
            Unit::Archer => [32, 0],
            Unit::WhiteMage => [32, 32],
            Unit::RedMage => [0, 32],
            Unit::OrcSword => [64, 0],
            Unit::OrcAxe => [96, 0],
            Unit::GoblinFighter => [64, 32],
            Unit::GoblinArcher => [96, 32],
            Unit::Squirrel => [64, 64],
            Unit::Spider => [0, 128],
            Unit::Bat => [0, 96],
            Unit::Ghost => [32, 128],
            Unit::Skeleton1 => [32, 64],
            Unit::Skeleton2 => [32, 96],
            Unit::Necromancer => [0, 64]
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Deserialize, Serialize)]
pub enum ClosedDoor {
    Gray,
    Green,
    Brown,
    Blue
}

impl ClosedDoor {
    pub fn get_coords(self) -> [usize; 2] {
        match self {
            ClosedDoor::Gray => [0, 288],
            ClosedDoor::Green => [0, 320],
            ClosedDoor::Brown => [64, 320],
            ClosedDoor::Blue => [0, 352]
        }
    }
}