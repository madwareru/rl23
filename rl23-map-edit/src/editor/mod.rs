use std::path::PathBuf;
use egui::{Align2, Context, Ui};
use retro_blit::{
    rendering::blittable::BlitBuilder,
    rendering::BlittableSurface,
    window::{KeyCode, RetroBlitContext, WindowMode}
};
use retro_blit::rendering::shapes::fill_rectangle;
use rl23_map_format::{MapEntity, TerrainKind, TilingInfo, Unit, WallKind, WangEncoding};
use crate::editor::tool::EditorTool;

const SCROLL_SPEED: f32 = 512.0;
const TILES_BYTES: &[u8] = include_bytes!("../../../assets/tiles.im256");

mod tool;

struct EditorApp {
    palette: Vec<[u8; 3]>,
    sprite_sheet: BlittableSurface,
    file_path: PathBuf,
    map_info: rl23_map_format::MapInfo,
    current_terrain_kind: TerrainKind,
    current_wall_kind: Option<WallKind>,
    current_entity_kind: Option<MapEntity>,
    current_tool: EditorTool,
    mouse_pressed: bool,
    camera_x: f32,
    camera_y: f32
}

pub fn open_for_edit(file_path: &PathBuf) {
    let file_path = file_path.clone();
    retro_blit::window::start(EditorApp::new(file_path));
}

impl EditorApp {
    pub fn new(file_path: PathBuf) -> Self {
        let (palette, sprite_sheet) = retro_blit::format_loaders::im_256::Image::load_from(TILES_BYTES).unwrap();
        let map_info = rl23_map_format::MapInfo::read_from_path(&file_path);
        Self {
            palette,
            sprite_sheet,
            file_path,
            map_info,
            current_tool: EditorTool::EditTerrain,
            current_terrain_kind: TerrainKind::Mud { offset: 0},
            current_wall_kind: Some(WallKind::Dirt),
            current_entity_kind: None,
            mouse_pressed: false,
            camera_x: 0.0,
            camera_y: 0.0
        }
    }

    fn handle_keyboard_input(&mut self, ctx: &mut RetroBlitContext, dt: f32) {
        if ctx.is_egui_wants_keyboard_input() {
            return;
        }

        if ctx.is_key_pressed(KeyCode::Left) {
            self.camera_x -= dt * SCROLL_SPEED;
        }
        if ctx.is_key_pressed(KeyCode::Right) {
            self.camera_x += dt * SCROLL_SPEED;
        }
        if ctx.is_key_pressed(KeyCode::Up) {
            self.camera_y -= dt * SCROLL_SPEED;
        }
        if ctx.is_key_pressed(KeyCode::Down) {
            self.camera_y += dt * SCROLL_SPEED;
        }
    }

    fn handle_mouse(&mut self, ctx: &mut RetroBlitContext) {
        if ctx.is_egui_area_under_pointer() {
            return;
        }

        let (coord_x, coord_y) = self.get_selection_coords(ctx);

        if self.mouse_pressed {
            self.put(coord_x, coord_y);
        }
    }

    fn get_selection_coords(&mut self, ctx: &mut RetroBlitContext) -> (i32, i32) {
        let (mouse_x, mouse_y) = ctx.get_mouse_pos();
        ((mouse_x + self.camera_x) as i32 / 32, (mouse_y + self.camera_y) as i32 / 32)
    }

    fn render_map(&mut self, ctx: &mut RetroBlitContext) {
        ctx.clear(0);

        let camera_x = self.camera_x as i32;
        let camera_y = self.camera_y as i32;

        // Render mud background. Any tile has it
        {
            for j in 0..self.map_info.height {
                for i in 0..self.map_info.width {
                    let terrain_kind = {
                        let idx = self.map_info.width * j + i;
                        match self.map_info.terrain_layer[idx] {
                            TerrainKind::Mud { offset } => TerrainKind::Mud { offset },
                            _ => TerrainKind::Mud { offset: 10 }
                        }
                    };
                    let tiling_info = terrain_kind.get_tiling_info();
                    match tiling_info {
                        TilingInfo::Mud(mud_terrain) => {
                            let [x, y] = mud_terrain.get_variant_coords_by_offset();
                            BlitBuilder::create(ctx, &self.sprite_sheet.with_color_key(0))
                                .with_source_subrect(x, y, 32, 32)
                                .with_dest_pos(
                                    (i as i32 * 32 - camera_x) as _,
                                    (j as i32 * 32 - camera_y) as _
                                ).blit();
                        }
                        _ => unreachable!()
                    }
                }
            }
        }

        // Render terrain
        {
            for j in 0..=self.map_info.height {
                for i in 0..=self.map_info.width {
                    let i_w = if i == 0 { i } else { i - 1 };
                    let i_e = if i >= self.map_info.width-1 { self.map_info.width-1 } else { i };

                    let j_n = if j == 0 { j } else { j - 1 };
                    let j_s = if j >= self.map_info.height-1 { self.map_info.height-1 } else { j };

                    for kind in [
                        TerrainKind::Sand,
                        TerrainKind::Dirt,
                        TerrainKind::Tile,
                        TerrainKind::BrightTile,
                        TerrainKind::MossTile,
                        TerrainKind::VibrantTile,
                        TerrainKind::Grass,
                        TerrainKind::CaveWater,
                        TerrainKind::Water,
                        TerrainKind::Lava
                    ] {
                        let mut encoding = WangEncoding {
                            north_east: false,
                            north_west: false,
                            south_east: false,
                            south_west: false
                        };
                        if self.map_info.terrain_layer[i_e + j_n * self.map_info.width] == kind {
                            encoding.north_east = true;
                        }
                        if self.map_info.terrain_layer[i_w + j_n * self.map_info.width] == kind {
                            encoding.north_west = true;
                        }
                        if self.map_info.terrain_layer[i_w + j_s * self.map_info.width] == kind {
                            encoding.south_west = true;
                        }
                        if self.map_info.terrain_layer[i_e + j_s * self.map_info.width] == kind {
                            encoding.south_east = true;
                        }

                        match kind.get_tiling_info() {
                            TilingInfo::Wang(wang) => {
                                if let Some([x, y]) = wang.get_final_coords(encoding) {
                                    BlitBuilder::create(ctx, &self.sprite_sheet.with_color_key(0))
                                        .with_source_subrect(x, y, 32, 32)
                                        .with_dest_pos((i as i32 * 32 - camera_x) as i16 - 16, (j as i32 * 32 - camera_y) as i16 -16)
                                        .blit();
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        // Render walls
        {
            for j in 0..=self.map_info.height {
                for i in 0..=self.map_info.width {
                    let i_w = if i == 0 { i } else { i - 1 };
                    let i_e = if i >= self.map_info.width-1 { self.map_info.width-1 } else { i };

                    let j_n = if j == 0 { j } else { j - 1 };
                    let j_s = if j >= self.map_info.height-1 { self.map_info.height-1 } else { j };

                    for kind in [
                        WallKind::Dirt,
                        WallKind::Bricks
                    ] {
                        let mut encoding = WangEncoding {
                            north_east: false,
                            north_west: false,
                            south_east: false,
                            south_west: false
                        };
                        if self.map_info.wall_layer[i_e + j_n * self.map_info.width] == Some(kind) {
                            encoding.north_east = true;
                        }
                        if self.map_info.wall_layer[i_w + j_n * self.map_info.width] == Some(kind) {
                            encoding.north_west = true;
                        }
                        if self.map_info.wall_layer[i_w + j_s * self.map_info.width] == Some(kind) {
                            encoding.south_west = true;
                        }
                        if self.map_info.wall_layer[i_e + j_s * self.map_info.width] == Some(kind) {
                            encoding.south_east = true;
                        }

                        match kind.get_tiling_info() {
                            TilingInfo::Wang(wang) => {
                                if let Some([x, y]) = wang.get_final_coords(encoding) {
                                    BlitBuilder::create(ctx, &self.sprite_sheet.with_color_key(0))
                                        .with_source_subrect(x, y, 32, 32)
                                        .with_dest_pos((i as i32 * 32 - camera_x) as i16 - 16, (j as i32 * 32 - camera_y) as i16 -16)
                                        .blit();
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }

        // Render entities
        {
            for (&idx, map_entity) in self.map_info.entity_layer.iter() {
                let coord_x = idx % self.map_info.width;
                let coord_y = idx / self.map_info.height;
                let (source_x, source_y) = match map_entity {
                    MapEntity::Door => (64, 352),
                    MapEntity::Unit(unit) => match unit {
                        Unit::Fighter => (0, 0),
                        Unit::Archer => (32,0),
                        Unit::WhiteMage => (32, 32),
                        Unit::RedMage => (0, 32),
                        Unit::OrcSword => (64, 0),
                        Unit::OrcAxe => (96, 0),
                        Unit::GoblinFighter => (64, 32),
                        Unit::GoblinArcher => (96, 32),
                        Unit::Squirrel => (64, 64),
                        Unit::Spider => (0, 128),
                        Unit::Bat => (0, 96),
                        Unit::Ghost => (32, 128),
                        Unit::Skeleton1 => (64, 32),
                        Unit::Skeleton2 => (96, 32),
                        Unit::Necromancer => (64, 0)
                    }
                };

                BlitBuilder::create(ctx, &self.sprite_sheet.with_color_key(0))
                    .with_source_subrect(source_x, source_y, 32, 32)
                    .with_dest_pos(
                        (coord_x as i32 * 32 - camera_x) as _,
                        (coord_y as i32 * 32 - camera_y) as _
                    ).blit();
            }
        }

        let min_x = (-camera_x) as i16;
        let min_y = (-camera_y) as i16;
        let max_x = (self.map_info.width as i32 * 32 - camera_x) as i16;
        let max_y = (self.map_info.height as i32 * 32 - camera_y) as i16;

        fill_rectangle(ctx, min_x - 16, min_y - 16, (self.map_info.width * 32 + 32) as u16, 16, 0);
        fill_rectangle(ctx, min_x - 16, max_y, (self.map_info.width * 32 + 32) as u16, 16, 0);
        fill_rectangle(ctx, min_x - 16, min_y, 16, self.map_info.height as u16 * 32, 0);
        fill_rectangle(ctx, max_x, min_y, 16, self.map_info.height as u16 * 32, 0);

        let (coord_x, coord_y) = self.get_selection_coords(ctx);

        if (0..self.map_info.width as i32).contains(&coord_x) &&
            (0..self.map_info.height as i32).contains(&coord_y) {
            BlitBuilder::create(ctx, &self.sprite_sheet.with_color_key(0))
                .with_source_subrect(0, 416, 32, 32)
                .with_dest_pos((coord_x * 32 - camera_x) as _, (coord_y * 32 - camera_y) as _)
                .blit();
        }
    }
}

impl retro_blit::window::ContextHandler for EditorApp {
    fn get_window_title(&self) -> &'static str {
        "editor"
    }

    fn get_window_mode(&self) -> WindowMode {
        WindowMode::Mode960x600
    }

    fn init(&mut self, ctx: &mut RetroBlitContext) {
        for i in 0..self.palette.len() {
            let [r, g, b] = self.palette[i];
            ctx.set_palette(i as _, [r, g, b]);
        }
    }

    fn on_mouse_down(&mut self, ctx: &mut RetroBlitContext, button_number: u8) {
        if ctx.is_egui_area_under_pointer() {
            return;
        }
        if button_number == 0 {
            self.mouse_pressed = true;
        }
    }

    fn on_mouse_up(&mut self, ctx: &mut RetroBlitContext, button_number: u8) {
        if ctx.is_egui_area_under_pointer() {
            return;
        }
        if button_number == 0 {
            self.mouse_pressed = false;
        }
    }

    fn update(&mut self, ctx: &mut RetroBlitContext, dt: f32) {
        self.handle_keyboard_input(ctx, dt);
        self.handle_mouse(ctx);
        self.render_map(ctx);
    }

    fn egui(&mut self, ctx: &mut RetroBlitContext, egui_ctx: Context) {
        egui::Window::new("general")
            .default_width(130.0)
            .resizable(false)
            .anchor(Align2::LEFT_TOP, [0.0, 0.0])
            .show(&egui_ctx, |ui: &mut Ui| {
                ui.radio_value(&mut self.current_tool, EditorTool::EditTerrain, "Terrain");
                ui.radio_value(&mut self.current_tool, EditorTool::EditEntities, "Entities");
                ui.radio_value(&mut self.current_tool, EditorTool::EditWalls, "Walls");

                ui.separator();
                if ui.button("Save").clicked() {
                    self.map_info.save_to_path(&self.file_path);
                }

                ui.separator();
                if ui.button("Quit").clicked() {
                    ctx.quit();
                }
            });

        egui::Window::new("tool")
            .default_width(130.0)
            .resizable(false)
            .anchor(Align2::RIGHT_TOP, [0.0, 0.0])
            .show(&egui_ctx, |ui: &mut Ui| {
                match self.current_tool {
                    EditorTool::EditTerrain => {
                        if ui.add(egui::RadioButton::new(
                            match self.current_terrain_kind {
                                TerrainKind::Mud { .. } => true,
                                _ => false
                            },
                            "Mud"
                        )).clicked() {
                            match self.current_terrain_kind {
                                TerrainKind::Mud { .. } => { },
                                _ => {
                                    self.current_terrain_kind = TerrainKind::Mud { offset: 0 }
                                }
                            }
                        }
                        ui.radio_value(&mut self.current_terrain_kind, TerrainKind::Sand, "Sand");
                        ui.radio_value(&mut self.current_terrain_kind, TerrainKind::Dirt, "Dirt");
                        ui.radio_value(&mut self.current_terrain_kind, TerrainKind::Grass, "Grass");
                        ui.radio_value(&mut self.current_terrain_kind, TerrainKind::Water, "Water");
                        ui.radio_value(&mut self.current_terrain_kind, TerrainKind::CaveWater, "CaveWater");
                        ui.radio_value(&mut self.current_terrain_kind, TerrainKind::Lava, "Lava");
                        ui.radio_value(&mut self.current_terrain_kind, TerrainKind::Tile, "Tile");
                        ui.radio_value(&mut self.current_terrain_kind, TerrainKind::BrightTile, "BrightTile");
                        ui.radio_value(&mut self.current_terrain_kind, TerrainKind::MossTile, "MossTile");
                        ui.radio_value(&mut self.current_terrain_kind, TerrainKind::VibrantTile, "VibrantTile");
                    }
                    EditorTool::EditWalls => {
                        ui.radio_value(&mut self.current_wall_kind, None, "None");
                        ui.radio_value(&mut self.current_wall_kind, Some(WallKind::Dirt), "Dirt");
                        ui.radio_value(&mut self.current_wall_kind, Some(WallKind::Bricks), "Bricks");
                    }
                    EditorTool::EditEntities => {
                        ui.radio_value(&mut self.current_entity_kind, None, "None");
                        ui.radio_value(&mut self.current_entity_kind, Some(MapEntity::Door), "Door");
                        if ui.add(egui::RadioButton::new(
                            match self.current_entity_kind {
                                Some(MapEntity::Unit(_)) => true,
                                _ => false
                            },
                            "Unit"
                        )).clicked() {
                            match self.current_entity_kind {
                                Some(MapEntity::Unit(_)) => { },
                                _ => {
                                    self.current_entity_kind = Some(MapEntity::Unit(Unit::Fighter))
                                }
                            }
                        }
                    }
                }
            });

        match self.current_entity_kind {
            Some(MapEntity::Unit(unit)) => {
                egui::Window::new("unit")
                    .default_width(130.0)
                    .resizable(false)
                    .anchor(Align2::CENTER_BOTTOM, [0.0, 0.0])
                    .show(&egui_ctx, |ui: &mut Ui| {
                        let mut unit = unit;
                        ui.vertical(|ui: &mut Ui| {
                            ui.horizontal(|ui: &mut Ui| {
                                ui.radio_value(&mut unit, Unit::Fighter, "Fighter");
                                ui.radio_value(&mut unit, Unit::Archer, "Archer");
                                ui.radio_value(&mut unit, Unit::WhiteMage, "WhiteMage");
                                ui.radio_value(&mut unit, Unit::RedMage, "RedMage");
                            });
                            ui.horizontal(|ui: &mut Ui| {
                                ui.radio_value(&mut unit, Unit::OrcSword, "OrcSword");
                                ui.radio_value(&mut unit, Unit::OrcAxe, "OrcAxe");
                                ui.radio_value(&mut unit, Unit::GoblinFighter, "GoblinFighter");
                                ui.radio_value(&mut unit, Unit::GoblinArcher, "GoblinArcher");
                            });
                            ui.horizontal(|ui: &mut Ui| {
                                ui.radio_value(&mut unit, Unit::Necromancer, "Necromancer");
                                ui.radio_value(&mut unit, Unit::Skeleton1, "Skeleton1");
                                ui.radio_value(&mut unit, Unit::Skeleton2, "Skeleton2");
                                ui.radio_value(&mut unit, Unit::Spider, "Spider");
                            });
                            ui.horizontal(|ui: &mut Ui| {
                                ui.radio_value(&mut unit, Unit::Bat, "Bat");
                                ui.radio_value(&mut unit, Unit::Ghost, "Ghost");
                                ui.radio_value(&mut unit, Unit::Squirrel, "Squirrel");
                            });
                        });
                        self.current_entity_kind = Some(MapEntity::Unit(unit));
                    });
            },
            _ => {}
        }
    }
}