use rand::Rng;
use rl23_map_format::{TerrainKind};
use crate::editor::EditorApp;

#[derive(Copy, Clone, PartialEq)]
pub enum EditorTool {
    EditTerrain,
    EditEntities,
    EditWalls
}

impl EditorApp {
    pub fn put(&mut self, x: i32, y: i32) {
        if x < 0 || y < 0 {
            return;
        }
        let x = x as usize;
        let y = y as usize;

        if x >= self.map_info.width {
            return;
        }

        if y >= self.map_info.height {
            return;
        }

        let idx = y * self.map_info.width + x;

        match self.current_tool {
            EditorTool::EditTerrain => {
                self.map_info.terrain_layer[idx] = self.current_terrain_kind;
                match self.current_terrain_kind {
                    TerrainKind::Mud { .. } => {
                        let mut rng = rand::thread_rng();
                        self.current_terrain_kind = TerrainKind::Mud {
                            offset: rng.gen_range(0..12)
                        };
                    }
                    _ => {}
                }
            }
            EditorTool::EditEntities => {
                match self.map_info.entity_layer.get(&idx) {
                    None => {
                        match self.current_entity_kind {
                            None => {}
                            Some(map_entity) => {
                                self.map_info.entity_layer.insert(idx, map_entity);
                            }
                        }
                    }
                    Some(&_) => {
                        match self.current_entity_kind {
                            None => {
                                self.map_info.entity_layer.remove(&idx);
                            }
                            Some(map_entity) => {
                                self.map_info.entity_layer.insert(idx, map_entity);
                            }
                        }
                    }
                }
            }
            EditorTool::EditWalls => {
                self.map_info.wall_layer[idx] = self.current_wall_kind;
            }
        }
    }
}