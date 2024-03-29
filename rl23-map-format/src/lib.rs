use std::{
    fs::{File},
    io::{Read},
    path::PathBuf
};
use std::collections::HashMap;
use std::hash::Hash;
use std::ops::{Deref, DerefMut};
use egui::{Ui};
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
    #[serde(default)]
    pub id_generator: IdGenerator,
    pub width: usize,
    pub height: usize,
    pub terrain_layer: Vec<TerrainKind>,
    #[serde(default)]
    pub gatherable_layer: HashMap<usize, GatherableItem>,
    #[serde(default)]
    pub entity_layer: HashMap<usize, MapEntity>,
    #[serde(default)]
    pub entity_data_layer: HashMap<usize, EntityComponentDataList>,
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
            id_generator: Default::default(),
            width,
            height,
            terrain_layer,
            gatherable_layer: Default::default(),
            entity_layer: Default::default(),
            entity_data_layer: Default::default(),
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

#[derive(Clone, Default, Debug, PartialEq, Deserialize, Serialize)]
pub struct IdGenerator {
    next_id: u64
}
impl IdGenerator {
    pub fn generate(&mut self) -> u64 {
        let id = self.next_id;
        self.next_id += 1;
        id
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Deserialize, Serialize)]
pub enum WallKind {
    Dirt,
    Bricks,
    Wood
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
            WallKind::Wood => TilingInfo::Wang(WangTerrain{
                x_offset: 20 * 32,
                y_offset: 12 * 32
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
    Decor(Decor),
    Unit(Unit),
    Tree(Tree),
    Loot,
    Logic
}

#[derive(Copy, Clone)]
pub struct EntityDrawCommand {
    pub coords: [u16; 2],
    pub size: [u16; 2],
    pub draw_offset: [i16; 2],
    pub drawing_layer: i8,
    pub blocks_tiles_above: u8,
}

impl Default for EntityDrawCommand {
    fn default() -> Self {
        Self {
            coords: [0, 0],
            size: [32, 32],
            draw_offset: [-2, -7],
            drawing_layer: 0,
            blocks_tiles_above: 0
        }
    }
}

impl MapEntity {
    pub fn get_draw_command(self) -> EntityDrawCommand {
        match self {
            MapEntity::Door => EntityDrawCommand {
                coords: [64, 480],
                draw_offset: [0, 0],
                ..Default::default()
            } ,
            MapEntity::ClosedDoor(closed_door) => closed_door.get_draw_command(),
            MapEntity::Decor(decor) => decor.get_draw_command(),
            MapEntity::Unit(unit) => unit.get_draw_command(),
            MapEntity::Tree(tree) => tree.get_draw_command(),
            MapEntity::Loot => EntityDrawCommand {
                coords: [512, 64],
                drawing_layer: -2,
                ..Default::default()
            } ,
            MapEntity::Logic => EntityDrawCommand { coords: [512, 256], ..Default::default() },
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Deserialize, Serialize)]
pub enum Unit {
    Leshy,
    MushroomMan,
    DarkWolf,
    Wolf,
    RogueKnife,
    RogueAxe,
    SnakeHuge,
    Snake,
    Squirrel,
    Stump,
    Czort,
    Imp,
    Spider,
    Bat,
    Ghost,
    Skeleton1,
    Skeleton2,
    Necromancer,
    DarkVigilante,
    DarkWarlord,
    Volkolak,
    Gorynich,
    Rusalka,
    Vodyanoy,
    Liho,
    Polevik,
    Poludenniza,
    PeasantMale1,
    PeasantMale2,
    PeasantMale3,
    PeasantMale4,
    PeasantFemale1,
    PeasantFemale2,
    PeasantFemale3,
    PeasantFemale4,
    PeasantFemale5,
    PeasantFemale6,
    PeasantFemale7,
    PeasantFemale8,
    PeasantFighter,
    PeasantArcher,
    SorcererWhite,
    SorcererRed
}

impl Unit {
    pub fn get_draw_command(self) -> EntityDrawCommand {
        match self {
            Unit::Leshy => EntityDrawCommand{
                coords: [0, 0],
                draw_offset: [0, -9],
                ..Default::default()
            },
            Unit::MushroomMan => EntityDrawCommand{ coords: [32, 0], ..Default::default() },
            Unit::DarkWolf => EntityDrawCommand{ coords: [32, 32], ..Default::default() },
            Unit::Wolf => EntityDrawCommand{ coords: [0, 32], ..Default::default() },
            Unit::RogueKnife => EntityDrawCommand{ coords: [64, 0], ..Default::default() },
            Unit::RogueAxe => EntityDrawCommand{ coords: [96, 0], ..Default::default() },
            Unit::SnakeHuge => EntityDrawCommand{ coords: [64, 32], ..Default::default() },
            Unit::Snake => EntityDrawCommand{ coords: [96, 32], ..Default::default() },
            Unit::Squirrel => EntityDrawCommand{ coords: [64, 64], ..Default::default() },
            Unit::Spider => EntityDrawCommand{ coords: [0, 128], ..Default::default() },
            Unit::Bat => EntityDrawCommand{ coords: [0, 96], ..Default::default() },
            Unit::Ghost => EntityDrawCommand{ coords: [32, 128], ..Default::default() },
            Unit::Skeleton1 => EntityDrawCommand{ coords: [32, 64], ..Default::default() },
            Unit::Skeleton2 => EntityDrawCommand{ coords: [32, 96], ..Default::default() },
            Unit::Necromancer => EntityDrawCommand{ coords: [0, 64], ..Default::default() },
            Unit::Stump => EntityDrawCommand{ coords: [96, 64], ..Default::default() },
            Unit::Czort => EntityDrawCommand{ coords: [96, 96], ..Default::default() },
            Unit::Imp => EntityDrawCommand{ coords: [96, 128], ..Default::default() },
            Unit::DarkVigilante => EntityDrawCommand{ coords: [0, 160], ..Default::default() },
            Unit::DarkWarlord => EntityDrawCommand{ coords: [32, 160], ..Default::default() },
            Unit::PeasantMale1 => EntityDrawCommand{ coords: [0, 288], ..Default::default() },
            Unit::PeasantMale2 => EntityDrawCommand{ coords: [32, 288], ..Default::default() },
            Unit::PeasantMale3 => EntityDrawCommand{ coords: [64, 288], ..Default::default() },
            Unit::PeasantMale4 => EntityDrawCommand{ coords: [96, 288], ..Default::default() },
            Unit::PeasantFemale1 => EntityDrawCommand{ coords: [0, 320], ..Default::default() },
            Unit::PeasantFemale2 => EntityDrawCommand{ coords: [32, 320], ..Default::default() },
            Unit::PeasantFemale3 => EntityDrawCommand{ coords: [64, 320], ..Default::default() },
            Unit::PeasantFemale4 => EntityDrawCommand{ coords: [96, 320], ..Default::default() },
            Unit::PeasantFemale5 => EntityDrawCommand{ coords: [0, 352], ..Default::default() },
            Unit::PeasantFemale6 => EntityDrawCommand{ coords: [32, 352], ..Default::default() },
            Unit::PeasantFemale7 => EntityDrawCommand{ coords: [64, 352], ..Default::default() },
            Unit::PeasantFemale8 => EntityDrawCommand{ coords: [96, 352], ..Default::default() },
            Unit::PeasantFighter => EntityDrawCommand{ coords: [0, 384], ..Default::default() },
            Unit::PeasantArcher => EntityDrawCommand{ coords: [32, 384], ..Default::default() },
            Unit::SorcererRed => EntityDrawCommand{ coords: [64, 384], ..Default::default() },
            Unit::SorcererWhite => EntityDrawCommand{ coords: [96, 384], ..Default::default() },
            Unit::Volkolak => EntityDrawCommand{
                coords: [64, 96],
                size: [32, 64],
                draw_offset: [0, -39],
                drawing_layer: 0,
                blocks_tiles_above: 1
            },
            Unit::Gorynich => EntityDrawCommand{
                coords: [64, 160],
                size: [64, 64],
                draw_offset: [-20, -39],
                drawing_layer: 0,
                blocks_tiles_above: 1
            },
            Unit::Rusalka => EntityDrawCommand{ coords: [0, 192], ..Default::default() },
            Unit::Vodyanoy => EntityDrawCommand{ coords: [32, 192], ..Default::default() },
            Unit::Liho => EntityDrawCommand{
                coords: [0, 224],
                draw_offset: [-2, -12],
                ..Default::default()
            },
            Unit::Poludenniza => EntityDrawCommand{ coords: [32, 224], ..Default::default() },
            Unit::Polevik => EntityDrawCommand{ coords: [64, 224], ..Default::default() },
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Deserialize, Serialize)]
pub enum Decor {
    Bed1GreenLeft,
    Bed1GreenRight,
    Bed2GreenLeft,
    Bed2GreenRight,

    Bed1BlueLeft,
    Bed1BlueRight,
    Bed2BlueLeft,
    Bed2BlueRight,

    Bed1WhiteLeft,
    Bed1WhiteRight,
    Bed2WhiteLeft,
    Bed2WhiteRight,

    TableGreen,
    TableBlue,
    TableBlack,

    OvenLeft,
    OvenRight,

    Closet,
    Dresser1,
    Dresser2
}

impl Decor {
    pub fn get_draw_command(self) -> EntityDrawCommand {
        match self {
            Decor::Bed1GreenLeft => EntityDrawCommand {
                coords: [832, 192],
                size: [64, 32],
                draw_offset: [0, 0],
                drawing_layer: 0,
                blocks_tiles_above: 0
            },
            Decor::Bed1GreenRight => EntityDrawCommand {
                coords: [960, 192],
                size: [64, 32],
                draw_offset: [-32, 0],
                drawing_layer: 0,
                blocks_tiles_above: 0
            },
            Decor::Bed2GreenLeft => EntityDrawCommand {
                coords: [768, 224],
                size: [64, 32],
                draw_offset: [0, 0],
                drawing_layer: 0,
                blocks_tiles_above: 0
            },
            Decor::Bed2GreenRight => EntityDrawCommand {
                coords: [768, 256],
                size: [64, 32],
                draw_offset: [-32, 0],
                drawing_layer: 0,
                blocks_tiles_above: 0
            },
            Decor::Bed1BlueLeft => EntityDrawCommand {
                coords: [832, 256],
                size: [64, 32],
                draw_offset: [0, 0],
                drawing_layer: 0,
                blocks_tiles_above: 0
            },
            Decor::Bed1BlueRight => EntityDrawCommand {
                coords: [960, 256],
                size: [64, 32],
                draw_offset: [-32, 0],
                drawing_layer: 0,
                blocks_tiles_above: 0
            },
            Decor::Bed2BlueLeft => EntityDrawCommand {
                coords: [768, 192],
                size: [64, 32],
                draw_offset: [0, 0],
                drawing_layer: 0,
                blocks_tiles_above: 0
            },
            Decor::Bed2BlueRight => EntityDrawCommand {
                coords: [768, 288],
                size: [64, 32],
                draw_offset: [-32, 0],
                drawing_layer: 0,
                blocks_tiles_above: 0
            },
            Decor::Bed1WhiteLeft => EntityDrawCommand {
                coords: [832, 224],
                size: [64, 32],
                draw_offset: [0, 0],
                drawing_layer: 0,
                blocks_tiles_above: 0
            },
            Decor::Bed1WhiteRight => EntityDrawCommand {
                coords: [960, 224],
                size: [64, 32],
                draw_offset: [-32, 0],
                drawing_layer: 0,
                blocks_tiles_above: 0
            },
            Decor::Bed2WhiteLeft => EntityDrawCommand {
                coords: [896, 288],
                size: [64, 32],
                draw_offset: [0, 0],
                drawing_layer: 0,
                blocks_tiles_above: 0
            },
            Decor::Bed2WhiteRight => EntityDrawCommand {
                coords: [960, 288],
                size: [64, 32],
                draw_offset: [-32, 0],
                drawing_layer: 0,
                blocks_tiles_above: 0
            },
            Decor::TableGreen => EntityDrawCommand {
                coords: [896, 192],
                size: [64, 32],
                draw_offset: [0, 0],
                drawing_layer: 0,
                blocks_tiles_above: 0
            },
            Decor::TableBlue => EntityDrawCommand {
                coords: [896, 224],
                size: [64, 32],
                draw_offset: [0, 0],
                drawing_layer: 0,
                blocks_tiles_above: 0
            },
            Decor::TableBlack => EntityDrawCommand {
                coords: [896, 256],
                size: [64, 32],
                draw_offset: [0, 0],
                drawing_layer: 0,
                blocks_tiles_above: 0
            },
            Decor::OvenLeft => EntityDrawCommand {
                coords: [832, 320],
                size: [64, 64],
                draw_offset: [0, -40],
                drawing_layer: 0,
                blocks_tiles_above: 0
            },
            Decor::OvenRight => EntityDrawCommand {
                coords: [960, 320],
                size: [64, 64],
                draw_offset: [-32, -40],
                drawing_layer: 0,
                blocks_tiles_above: 0
            },
            Decor::Closet => EntityDrawCommand {
                coords: [896, 320],
                size: [42, 42],
                draw_offset: [-5, -26],
                drawing_layer: 0,
                blocks_tiles_above: 0
            },
            Decor::Dresser1 => EntityDrawCommand {
                coords: [832, 288],
                size: [32, 32],
                draw_offset: [0, -16],
                drawing_layer: 0,
                blocks_tiles_above: 0
            },
            Decor::Dresser2 => EntityDrawCommand {
                coords: [864, 288],
                size: [32, 32],
                draw_offset: [0, -16],
                drawing_layer: 0,
                blocks_tiles_above: 0
            },
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Deserialize, Serialize)]
pub enum Tree {
    Pine1,
    Pine2,
    Oak,
    Birch,
    Pine1Cursed,
    Pine2Cursed,
    OakCursed,
    BirchCursed
}

impl Tree {
    pub fn get_draw_command(self) -> EntityDrawCommand {
        match self {
            Tree::Pine1 => EntityDrawCommand {
                coords: [736, 0],
                size: [96, 96],
                draw_offset: [-34, -64],
                drawing_layer: 0,
                blocks_tiles_above: 2
            },
            Tree::Pine2 => EntityDrawCommand {
                coords: [832, 24],
                size: [32, 62],
                draw_offset: [-2, -40],
                drawing_layer: 0,
                blocks_tiles_above: 1
            },
            Tree::Oak => EntityDrawCommand {
                coords: [864, 0],
                size: [96, 96],
                draw_offset: [-32, -64],
                drawing_layer: 0,
                blocks_tiles_above: 2
            },
            Tree::Birch => EntityDrawCommand {
                coords: [986, 0],
                size: [38, 84],
                draw_offset: [-8, -64],
                drawing_layer: 0,
                blocks_tiles_above: 2
            },
            Tree::Pine1Cursed => EntityDrawCommand {
                coords: [736, 96],
                size: [96, 96],
                draw_offset: [-34, -64],
                drawing_layer: 0,
                blocks_tiles_above: 2
            },
            Tree::Pine2Cursed => EntityDrawCommand {
                coords: [832, 120],
                size: [32, 62],
                draw_offset: [-2, -40],
                drawing_layer: 0,
                blocks_tiles_above: 1
            },
            Tree::OakCursed => EntityDrawCommand {
                coords: [864, 96],
                size: [96, 96],
                draw_offset: [-32, -64],
                drawing_layer: 0,
                blocks_tiles_above: 2
            },
            Tree::BirchCursed => EntityDrawCommand {
                coords: [986, 96],
                size: [38, 84],
                draw_offset: [-8, -64],
                drawing_layer: 0,
                blocks_tiles_above: 2
            },
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
    pub fn get_draw_command(self) -> EntityDrawCommand {
        match self {
            ClosedDoor::Gray => EntityDrawCommand {
                coords: [0, 416],
                draw_offset: [0, 0],
                ..Default::default()
            },
            ClosedDoor::Green => EntityDrawCommand {
                coords: [0, 448],
                draw_offset: [0, 0],
                ..Default::default()
            },
            ClosedDoor::Brown => EntityDrawCommand {
                coords: [64, 448],
                draw_offset: [0, 0],
                ..Default::default()
            },
            ClosedDoor::Blue => EntityDrawCommand {
                coords: [0, 480],
                draw_offset: [0, 0],
                ..Default::default()
            },
        }
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Deserialize, Serialize)]
pub enum GatherableItem {
    Mushroom(usize),
    Wheat
}

impl GatherableItem {
    pub fn get_coords(self) -> [usize; 2] {
        match self {
            GatherableItem::Mushroom(num) => {
                let num = num % 10;
                [
                    512 + 32 * (num % 5),
                    32 * (num / 5)
                ]
            }
            GatherableItem::Wheat => [
                544,
                64
            ]
        }
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub struct EntityComponentDataList {
    id: u64,
    components: Vec<EntityComponentData>
}

impl EntityComponentDataList {
    pub fn create(id: u64) -> Self {
        Self {
            id,
            components: Vec::new()
        }
    }
    pub fn id(&self) -> impl Hash + Sized {
        self.id
    }
    pub fn remove(&mut self, idx: usize) {
        self.components.remove(idx);
    }
    pub fn push(&mut self, data: EntityComponentData) {
        self.components.push(data);
    }
}

impl Deref for EntityComponentDataList {
    type Target = [EntityComponentData];
    fn deref(&self) -> &Self::Target {
        &self.components
    }
}

impl DerefMut for EntityComponentDataList {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.components
    }
}

#[derive(Clone, Debug, PartialEq, Deserialize, Serialize)]
pub enum EntityComponentData {
    SpawnRandomUnit(SpawnRandomUnit)
}

impl EntityComponentData {
    pub fn draw_context_menu(map_entity: MapEntity, id_generator: &mut IdGenerator, ui: &mut egui::Ui) -> Option<Self> {
        let mut result = None;
        ui.menu_button("+", |ui: &mut egui::Ui| {
            macro_rules! menu_entry(
                ($type_name:ident as $name: literal) => {
                    if $type_name::is_applicable_for_enitity_type(map_entity) {
                        if ui.button($name).clicked() {
                            result = Some(Self::$type_name($type_name::make_default(id_generator)));
                        }
                    }
                }
            );
            menu_entry!(SpawnRandomUnit as "Spawn Random Unit");
        });
        result
    }

    pub fn draw_egui(&mut self, id_generator: &mut IdGenerator, ui: &mut egui::Ui) -> bool {
        let mut delete = false;
        match self {
            EntityComponentData::SpawnRandomUnit(spawn_random_unit) =>
                spawn_random_unit.draw_egui(id_generator, ui)
        }
        if ui.button("DELETE").clicked() {
            delete = true;
        }
        ui.separator();
        !delete
    }
}

pub trait EntityComponentDataImpl: Clone + Default {
    fn is_applicable_for_enitity_type(map_entity: MapEntity) -> bool;
    fn draw_egui(&mut self, id_generator: &mut IdGenerator, ui: &mut egui::Ui);
    fn make_default(id_generator: &mut IdGenerator) -> Self;
}

#[derive(Default, Copy, Clone, PartialEq, Debug, Deserialize, Serialize)]
pub struct SpawnRandomUnit {
    pub min_level: i32,
    pub max_level: i32
}

impl EntityComponentDataImpl for SpawnRandomUnit {
    fn draw_egui(&mut self, _id_generator: &mut IdGenerator, ui: &mut Ui) {
            ui.label(padded_str("Spawn Random Unit"));
            ui.add(egui::DragValue::new(&mut self.min_level).prefix("min_level: ").speed(1.0));
            ui.add(egui::DragValue::new(&mut self.max_level).prefix("max_level: ").speed(1.0));
    }

    fn is_applicable_for_enitity_type(map_entity: MapEntity) -> bool {
        match map_entity {
            MapEntity::Logic => true,
            _ => false
        }
    }

    fn make_default(_id_generator: &mut IdGenerator) -> Self {
        Default::default()
    }
}

fn padded_str(s: &str) -> String {
    format!("{:<35}", s)
}