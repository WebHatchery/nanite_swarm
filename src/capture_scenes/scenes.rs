//! Screenshot scene staging implementation.

use crate::state::LaunchSequence;
use crate::{engine, state, Game, GamePhase};

mod arrival;
mod ending;
mod factory;
mod industrial;
mod interplanetary;
mod menu;
mod power;
mod reset;
mod terrain;
mod world;

impl Game {
    /// Seed a specific scene for the screenshot harness.
    pub fn begin_capture_scene(&mut self, scene: &str) {
        self.reset_capture_scene();
        if self.stage_menu_capture_scene(scene) {
            return;
        }
        if self.stage_industrial_capture_scene(scene) {
            return;
        }
        if self.stage_world_capture_scene(scene) {
            return;
        }
        match scene {
            "research" | "mobile_research" | "mobile_research_narrow" => {
                self.stage_research_scene()
            }
            "feature_inspection" | "mobile_feature_inspection" => {
                self.stage_feature_inspection_scene()
            }
            "partial_feature_inspection" => self.stage_partial_feature_inspection_scene(),
            "iron_belt_inspection" => self.stage_iron_belt_inspection_scene(),
            "iron_belt_drill_inspection" => self.stage_iron_belt_drill_inspection_scene(),
            "mountain_wind_inspection" | "mobile_mountain_wind_inspection" => {
                self.stage_mountain_wind_inspection_scene()
            }
            "forest_harvester_inspection" | "mobile_forest_harvester_inspection" => {
                self.stage_forest_harvester_inspection_scene()
            }
            "filter_forest_inspection" | "mobile_filter_forest_inspection" => {
                self.stage_filter_forest_inspection_scene()
            }
            "acid_sea_inspection" => self.stage_acid_sea_inspection_scene(),
            "ring_shadow_inspection" => self.stage_ring_shadow_inspection_scene(),
            "ice_rift_inspection" => self.stage_ice_rift_inspection_scene(),
            "landing_crater_inspection" => self.stage_landing_crater_inspection_scene(),
            // The two beats of a launch worth looking at in a still frame: the
            // ship clearing the world it was built on, and the one it reaches.
            "launch" | "arrival" | "mobile_launch" | "mobile_arrival" => {
                self.campaign.colonize(1);
                self.campaign.travel_to(1);
                let mut sequence = LaunchSequence::new_with_data(self.data.clone(), 0, 1);
                sequence.advance(if matches!(scene, "launch" | "mobile_launch") {
                    sequence.beat_start(state::LaunchBeat::Ascent) + 1.2
                } else {
                    sequence.beat_start(state::LaunchBeat::Arrival) + 1.5
                });
                self.launch = Some(sequence);
                self.phase = GamePhase::Launch;
            }
            "mobile_arrival_notice" => self.stage_arrival_notice_scene(),
            "venus" => self.stage_venus_scene(),
            "geothermal" => self.stage_geothermal_scene(),
            "fusion" => self.stage_fusion_scene(),
            "ending" | "mobile_ending_very_short" => self.stage_ending_scene(),
            "desktop_ending" => self.stage_ending_scene(),
            "interplanetary" => self.stage_interplanetary_scene(),
            "desktop_interplanetary_short" => {
                self.stage_interplanetary_scene();
            }
            "interplanetary_launch_confirm" => {
                self.stage_interplanetary_scene();
                self.launch_confirmation_target = Some(0);
            }
            "interplanetary_driver_offline" | "mobile_driver_offline" => {
                self.stage_interplanetary_scene();
                let driver = self
                    .campaign
                    .current()
                    .grid
                    .find_buildings(engine::BuildingType::MassDriver)
                    .first()
                    .copied();
                if let Some(driver) = driver {
                    if let Some(building) = self
                        .campaign
                        .current_mut()
                        .grid
                        .get_mut(driver)
                        .and_then(|tile| tile.building.as_mut())
                    {
                        building.standby = true;
                    }
                }
                if scene == "mobile_driver_offline" {
                    self.interplanetary_mobile_panel = state::InterplanetaryMobilePanel::Shipping;
                }
            }
            "interplanetary_pad_offline" => self.stage_interplanetary_pad_offline_scene(),
            "interplanetary_remote_orders" => self.stage_interplanetary_remote_orders_scene(),
            "interplanetary_core_stages" => self.stage_interplanetary_core_stages_scene(),
            "mobile_interplanetary_core_stages" => self.stage_interplanetary_core_stages_scene(),
            "mobile_remote_orders" => {
                self.stage_interplanetary_remote_orders_scene();
                self.interplanetary_mobile_panel = state::InterplanetaryMobilePanel::Shipping;
            }
            "mobile_remote_recovery" => self.stage_mobile_remote_recovery_scene(),
            "mobile_interplanetary_narrow" => self.stage_interplanetary_scene(),
            "mobile_shipping" => {
                self.stage_interplanetary_scene();
                self.interplanetary_mobile_panel = state::InterplanetaryMobilePanel::Shipping;
            }
            "mobile_map" => {
                self.stage_interplanetary_scene();
                self.interplanetary_mobile_panel = state::InterplanetaryMobilePanel::Worlds;
            }
            "interplanetary_rules" => {
                self.stage_interplanetary_scene();
                self.campaign.colonize(1);
                self.campaign.travel_to(1);
                self.campaign.current_mut().power_balance = -1.0;
            }
            "mobile_interplanetary_rules" => {
                self.stage_interplanetary_scene();
                self.campaign.colonize(1);
                self.campaign.travel_to(1);
                self.interplanetary_mobile_panel = state::InterplanetaryMobilePanel::Worlds;
            }
            "mobile_interplanetary_alert" => {
                self.stage_mobile_interplanetary_alert_scene();
            }
            "mobile_shipping_reserve" => {
                self.stage_interplanetary_scene();
                self.campaign.cycle_export_reserve_for(2);
                self.interplanetary_mobile_panel = state::InterplanetaryMobilePanel::Shipping;
            }
            _ => {
                // Default: jump straight into gameplay on the starting planet.
                self.phase = GamePhase::Playing;
            }
        }
    }
}
