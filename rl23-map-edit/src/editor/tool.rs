use egui::{Align2, Context, Ui};
use rand::Rng;
use retro_blit::window::RetroBlitContext;
use rl23_map_format::{ClosedDoor, EntityComponentData, GatherableItem, MapEntity, TerrainKind, Unit, WallKind};
use crate::editor::EditorApp;

#[derive(Copy, Clone, PartialEq)]
pub enum EditorTool {
    Terrain,
    Gatherables,
    Entities,
    EditEntities,
    Walls
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
            EditorTool::Terrain => {
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
            EditorTool::Entities => {
                match self.map_info.entity_layer.get(&idx) {
                    None => {
                        match self.current_entity_kind {
                            None => {}
                            Some(map_entity) => {
                                self.map_info.entity_layer.insert(idx, map_entity);
                                self.map_info.entity_data_layer.insert(idx, vec![]);
                            }
                        }
                    }
                    Some(&old_entity) => {
                        match self.current_entity_kind {
                            None => {
                                self.map_info.entity_layer.remove(&idx);
                                self.map_info.entity_data_layer.remove(&idx);
                                match self.current_edited_entity {
                                    Some(id) if id == idx => {
                                        self.current_edited_entity = None;
                                    },
                                    _ => {}
                                }
                            }
                            Some(map_entity) => {
                                if old_entity != map_entity {
                                    self.map_info.entity_layer.insert(idx, map_entity);
                                    self.map_info.entity_data_layer.insert(idx, vec![]);
                                }
                            }
                        }
                    }
                }
            }
            EditorTool::Walls => {
                self.map_info.wall_layer[idx] = self.current_wall_kind;
            }
            EditorTool::Gatherables => {
                match self.map_info.gatherable_layer.get(&idx) {
                    None => {
                        match self.current_gatherable_kind {
                            None => {}
                            Some(gatherable) => {
                                self.map_info.gatherable_layer.insert(idx, gatherable);
                            }
                        }
                    }
                    Some(&_) => {
                        match self.current_gatherable_kind {
                            None => {
                                self.map_info.gatherable_layer.remove(&idx);
                            }
                            Some(gatherable) => {
                                self.map_info.gatherable_layer.insert(idx, gatherable);
                            }
                        }
                    }
                }
            }
            EditorTool::EditEntities => {
                self.current_edited_entity = if self.map_info.entity_layer.contains_key(&idx) {
                    Some(idx)
                } else {
                    None
                };
            }
        }
    }
    pub fn tools_ui(&mut self, ctx: &mut RetroBlitContext, egui_ctx: &Context) {
        egui::Window::new("general")
            .default_width(130.0)
            .resizable(false)
            .anchor(Align2::LEFT_TOP, [0.0, 0.0])
            .show(&egui_ctx, |ui: &mut Ui| {
                ui.radio_value(&mut self.current_tool, EditorTool::Terrain, "Terrain");
                ui.radio_value(&mut self.current_tool, EditorTool::Gatherables, "Gatherables");
                ui.radio_value(&mut self.current_tool, EditorTool::Entities, "Entities");
                ui.radio_value(&mut self.current_tool, EditorTool::EditEntities, "Edit Entities");
                ui.radio_value(&mut self.current_tool, EditorTool::Walls, "Walls");

                ui.separator();
                if ui.button("Save").clicked() {
                    self.map_info.save_to_path(&self.file_path);
                }

                ui.separator();
                if ui.button("Quit").clicked() {
                    ctx.quit();
                }
            });

        let tool_title = match self.current_tool {
            EditorTool::Terrain => "Brush                  ",
            EditorTool::Gatherables => "Brush                  ",
            EditorTool::Entities => "Brush                  ",
            EditorTool::EditEntities => "Inspector            ",
            EditorTool::Walls => "Brush                  "
        };

        egui::Window::new(tool_title)
            .resizable(false)
            .anchor(Align2::RIGHT_TOP, [0.0, 0.0])
            .show(&egui_ctx, |ui: &mut Ui| {
                match self.current_tool {
                    EditorTool::Terrain => {
                        if ui.add(egui::RadioButton::new(
                            match self.current_terrain_kind {
                                TerrainKind::Mud { .. } => true,
                                _ => false
                            },
                            "Mud"
                        )).clicked() {
                            match self.current_terrain_kind {
                                TerrainKind::Mud { .. } => {},
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
                    EditorTool::Walls => {
                        ui.radio_value(&mut self.current_wall_kind, None, "None");
                        ui.radio_value(&mut self.current_wall_kind, Some(WallKind::Dirt), "Dirt");
                        ui.radio_value(&mut self.current_wall_kind, Some(WallKind::Bricks), "Bricks");
                        ui.radio_value(&mut self.current_wall_kind, Some(WallKind::Wood), "Wood");
                    }
                    EditorTool::Entities => {
                        ui.radio_value(&mut self.current_entity_kind, None, "None");
                        ui.radio_value(&mut self.current_entity_kind, Some(MapEntity::Door), "Door");

                        if ui.add(egui::RadioButton::new(
                            match self.current_entity_kind {
                                Some(MapEntity::ClosedDoor(_)) => true,
                                _ => false
                            },
                            "Closed Door"
                        )).clicked() {
                            match self.current_entity_kind {
                                Some(MapEntity::ClosedDoor(_)) => {},
                                _ => {
                                    self.current_entity_kind = Some(MapEntity::ClosedDoor(ClosedDoor::Gray))
                                }
                            }
                        }

                        if ui.add(egui::RadioButton::new(
                            match self.current_entity_kind {
                                Some(MapEntity::Unit(_)) => true,
                                _ => false
                            },
                            "Unit"
                        )).clicked() {
                            match self.current_entity_kind {
                                Some(MapEntity::Unit(_)) => {},
                                _ => {
                                    self.current_entity_kind = Some(MapEntity::Unit(Unit::Fighter))
                                }
                            }
                        }

                        ui.radio_value(&mut self.current_entity_kind, Some(MapEntity::Loot), "Loot");
                        ui.radio_value(&mut self.current_entity_kind, Some(MapEntity::Logic), "Logic");

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
                            Some(MapEntity::ClosedDoor(closed_door)) => {
                                egui::Window::new("door kind")
                                    .default_width(130.0)
                                    .resizable(false)
                                    .anchor(Align2::CENTER_BOTTOM, [0.0, 0.0])
                                    .show(&egui_ctx, |ui: &mut Ui| {
                                        let mut closed_door = closed_door;
                                        ui.vertical(|ui: &mut Ui| {
                                            ui.horizontal(|ui: &mut Ui| {
                                                ui.radio_value(&mut closed_door, ClosedDoor::Gray, "Gray");
                                                ui.radio_value(&mut closed_door, ClosedDoor::Green, "Green");
                                                ui.radio_value(&mut closed_door, ClosedDoor::Brown, "Brown");
                                                ui.radio_value(&mut closed_door, ClosedDoor::Blue, "Blue");
                                            });
                                        });
                                        self.current_entity_kind = Some(MapEntity::ClosedDoor(closed_door));
                                    });
                            },
                            _ => {}
                        }
                    }
                    EditorTool::Gatherables => {
                        ui.radio_value(&mut self.current_gatherable_kind, None, "None");
                        if ui.add(egui::RadioButton::new(
                            match self.current_gatherable_kind {
                                Some(GatherableItem::Mushroom(_)) => true,
                                _ => false
                            },
                            "Mushroom"
                        )).clicked() {
                            match self.current_gatherable_kind {
                                Some(GatherableItem::Mushroom(_)) => {},
                                _ => {
                                    self.current_gatherable_kind = Some(GatherableItem::Mushroom(0))
                                }
                            }
                        }
                        ui.radio_value(
                            &mut self.current_gatherable_kind, Some(GatherableItem::Wheat),
                            "Wheat"
                        );

                        match self.current_gatherable_kind {
                            Some(GatherableItem::Mushroom(offset)) => {
                                egui::Window::new("kind")
                                    .default_width(130.0)
                                    .resizable(false)
                                    .anchor(Align2::CENTER_BOTTOM, [0.0, 0.0])
                                    .show(&egui_ctx, |ui: &mut Ui| {
                                        let mut offset = offset;
                                        ui.vertical(|ui: &mut Ui| {
                                            ui.horizontal(|ui: &mut Ui| {
                                                ui.radio_value(&mut offset, 0, "Green");
                                                ui.radio_value(&mut offset, 1, "Blue");
                                                ui.radio_value(&mut offset, 2, "Red");
                                                ui.radio_value(&mut offset, 3, "Brown");
                                                ui.radio_value(&mut offset, 4, "Gray");
                                            });
                                        });
                                        ui.vertical(|ui: &mut Ui| {
                                            ui.horizontal(|ui: &mut Ui| {
                                                ui.radio_value(&mut offset, 5, "Green Sharp");
                                                ui.radio_value(&mut offset, 6, "Blue Sharp");
                                                ui.radio_value(&mut offset, 7, "Red Sharp");
                                                ui.radio_value(&mut offset, 8, "Brown Sharp");
                                                ui.radio_value(&mut offset, 9, "Gray Sharp");
                                            });
                                        });
                                        self.current_gatherable_kind = Some(GatherableItem::Mushroom(offset));
                                    });
                            }
                            Some(GatherableItem::Wheat) => {}
                            None => {}
                        }
                    }
                    EditorTool::EditEntities => {
                        if let Some(idx) = self.current_edited_entity {
                            egui::ScrollArea::vertical().auto_shrink([false, true]).show(ui, |ui: &mut Ui| {
                                let map_entity = self.map_info.entity_layer[&idx];
                                if !self.map_info.entity_data_layer.contains_key(&idx) {
                                    self.map_info.entity_data_layer.insert(idx, vec![]);
                                }
                                match self.map_info.entity_data_layer.get_mut(&idx) {
                                    Some(entries) => {
                                        let mut offset = 0;
                                        while offset < entries.len() {
                                            if entries[offset].draw_egui(ui) {
                                                offset += 1;
                                            } else {
                                                entries.remove(offset);
                                            }
                                        }
                                        if let Some(new_entry) = EntityComponentData::draw_context_menu(map_entity, ui) {
                                            entries.push(new_entry);
                                        }
                                    },
                                    None => unreachable!()
                                }
                            });
                        }
                    }
                }
            });
    }
}