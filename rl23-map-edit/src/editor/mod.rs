use std::path::PathBuf;
use egui::{Context, FontData, FontDefinitions, FontFamily};
use retro_blit::{
    rendering::blittable::BlitBuilder,
    rendering::BlittableSurface,
    window::{KeyCode, RetroBlitContext, WindowMode}
};
use retro_blit::rendering::shapes::fill_rectangle;
use rl23_map_format::{EntityDrawCommand, GatherableItem, MapEntity, TerrainKind, TilingInfo, WallKind, WangEncoding};
use crate::editor::tool::EditorTool;

const SCROLL_SPEED: f32 = 512.0;
const TILES_BYTES: &[u8] = include_bytes!("../../../assets/tiles.im256");
const JETBRAINS_MONO_FONT: &[u8] = include_bytes!("../../../assets/JetBrainsMono-Medium.ttf");

mod tool;

struct EditorApp {
    palette: Vec<[u8; 3]>,
    sprite_sheet: BlittableSurface,
    file_path: PathBuf,
    map_info: rl23_map_format::MapInfo,
    current_terrain_kind: TerrainKind,
    current_wall_kind: Option<WallKind>,
    current_gatherable_kind: Option<GatherableItem>,
    current_entity_kind: Option<MapEntity>,
    current_edited_entity: Option<usize>,
    current_tool: EditorTool,
    mouse_pressed: bool,
    camera_x: f32,
    camera_y: f32,
    entity_draw_queue: Vec<Vec<EntityDrawCommand>>
}

pub fn open_for_edit(file_path: &PathBuf) {
    let file_path = file_path.clone();
    retro_blit::window::start(EditorApp::new(file_path));
}

impl EditorApp {
    pub fn new(file_path: PathBuf) -> Self {
        let (mut palette, sprite_sheet) = retro_blit::format_loaders::im_256::Image::load_from(TILES_BYTES).unwrap();
        for pal in palette.iter_mut() {
            let r = pal[0] as f32;
            let g = pal[1] as f32;
            let b = pal[2] as f32;

            let luma = (0.2126 * r + 0.7152 * g + 0.0722 * b).clamp(0.0, 255.0);

            let r = (r * 0.8 + luma * 0.2 + 5.0 * (1.0 - b / 255.0)).clamp(0.0, 255.0) as u8;
            let g = (g * 0.8 + luma * 0.2 + 2.5 * (1.0 - b / 255.0)).clamp(0.0, 255.0) as u8;
            let b = (b * 0.8 + luma * 0.2).clamp(0.0, 255.0) as u8;

            *pal = [r, g, b];
        }

        let map_info = rl23_map_format::MapInfo::read_from_path(&file_path);
        let size = map_info.width * map_info.height;
        Self {
            palette,
            sprite_sheet,
            file_path,
            map_info,
            current_tool: EditorTool::Terrain,
            current_terrain_kind: TerrainKind::Mud { offset: 0},
            current_wall_kind: Some(WallKind::Dirt),
            current_gatherable_kind: None,
            current_entity_kind: None,
            mouse_pressed: false,
            camera_x: 0.0,
            camera_y: 0.0,
            current_edited_entity: None,
            entity_draw_queue: vec![Vec::with_capacity(4); size]
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

        if let Some((coord_x, coord_y)) = self.get_selection_coords(ctx) {
            if self.mouse_pressed {
                self.put(coord_x, coord_y);
            }
        }
    }

    fn get_edited_entity_coords(&mut self) -> Option<(i32, i32)> {
        if let EditorTool::EditEntities = self.current_tool {
            let idx = self.current_edited_entity?;
            let x = idx % self.map_info.width;
            let y = idx / self.map_info.width;
            Some((x as i32, y as i32))
        } else {
            None
        }
    }

    fn get_selection_coords(&mut self, ctx: &mut RetroBlitContext) -> Option<(i32, i32)> {
        let (mouse_x, mouse_y) = ctx.get_mouse_pos();
        let rx = mouse_x + self.camera_x;
        let ry = mouse_y + self.camera_y;
        if rx >= 0.0 && ry >= 0.0 {
            Some((rx as i32 / 32, ry as i32 / 32))
        } else {
            None
        }
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

        // Render gatherables
        {
            for (&idx, gatherable) in self.map_info.gatherable_layer.iter() {
                let coord_x = idx % self.map_info.width;
                let coord_y = idx / self.map_info.height;
                let [source_x, source_y] = gatherable.get_coords();

                BlitBuilder::create(ctx, &self.sprite_sheet.with_color_key(0))
                    .with_source_subrect(source_x, source_y, 32, 32)
                    .with_dest_pos(
                        (coord_x as i32 * 32 - camera_x) as _,
                        (coord_y as i32 * 32 - camera_y) as _
                    ).blit();
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
                        WallKind::Bricks,
                        WallKind::Wood
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
            for queue in self.entity_draw_queue.iter_mut() {
                queue.clear();
            }
            for (&idx, map_entity) in self.map_info.entity_layer.iter() {
                let command = map_entity.get_draw_command();
                self.entity_draw_queue[idx].push(command);
            }

            for (idx, command_queue) in self.entity_draw_queue.iter_mut().enumerate() {
                let coord_x = idx % self.map_info.width;
                let coord_y = idx / self.map_info.height;
                command_queue.sort_by(|lhs, rhs| lhs.drawing_layer.cmp(&rhs.drawing_layer));
                for cmd in command_queue.iter() {
                    let [source_x, source_y] = cmd.coords;

                    BlitBuilder::create(ctx, &self.sprite_sheet.with_color_key(0))
                        .with_source_subrect(
                            source_x as _, source_y as _, cmd.size[0] as _, cmd.size[1] as _
                        ).with_dest_pos(
                            (coord_x as i32 * 32 - camera_x) as i16 + cmd.draw_offset[0],
                            (coord_y as i32 * 32 - camera_y) as i16 + cmd.draw_offset[1]
                        ).blit();
                }
            }
        }

        // Draw selections
        {
            if let Some((entity_x, entity_y)) = self.get_edited_entity_coords() {
                BlitBuilder::create(ctx, &self.sprite_sheet.with_color_key(0))
                    .with_source_subrect(512, 320, 32, 32)
                    .with_dest_pos((entity_x * 32 - camera_x) as _, (entity_y * 32 - camera_y) as _)
                    .blit();
            }

            if let Some((coord_x, coord_y)) = self.get_selection_coords(ctx) {
                if (0..self.map_info.width as i32).contains(&coord_x) &&
                    (0..self.map_info.height as i32).contains(&coord_y) {
                    BlitBuilder::create(ctx, &self.sprite_sheet.with_color_key(0))
                        .with_source_subrect(512, 288, 32, 32)
                        .with_dest_pos((coord_x * 32 - camera_x) as _, (coord_y * 32 - camera_y) as _)
                        .blit();
                }
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
    }
}

impl retro_blit::window::ContextHandler for EditorApp {
    fn get_window_title(&self) -> &'static str {
        "editor"
    }

    fn get_window_mode(&self) -> WindowMode {
        WindowMode::Mode960x600
    }

    fn on_mouse_down(&mut self, ctx: &mut RetroBlitContext, button_number: u8) {
        if ctx.is_egui_area_under_pointer() {
            return;
        }
        if button_number == 0 {
            self.mouse_pressed = true;
        }
    }

    fn on_mouse_up(&mut self, _ctx: &mut RetroBlitContext, button_number: u8) {
        if button_number == 0 {
            self.mouse_pressed = false;
        }
    }

    fn init(&mut self, ctx: &mut RetroBlitContext) {
        for i in 0..self.palette.len() {
            let [r, g, b] = self.palette[i];
            ctx.set_palette(i as _, [r, g, b]);
        }

        let mut fonts = FontDefinitions::default();

        fonts.font_data
            .insert("JetBrains Mono".to_owned(), FontData::from_static(JETBRAINS_MONO_FONT));
        fonts.families
            .get_mut(&FontFamily::Proportional)
            .unwrap()
            .insert(0, "JetBrains Mono".to_owned());
        fonts.families
            .get_mut(&FontFamily::Monospace)
            .unwrap()
            .insert(0, "JetBrains Mono".to_owned());
        ctx.get_egui_ctx().set_fonts(fonts);
    }

    fn update(&mut self, ctx: &mut RetroBlitContext, dt: f32) {
        self.handle_keyboard_input(ctx, dt);
        self.handle_mouse(ctx);
        self.render_map(ctx);
    }

    fn egui(&mut self, ctx: &mut RetroBlitContext, egui_ctx: Context) {
        self.tools_ui(ctx, &egui_ctx)
    }
}