//! Solar System snapshot assembly and freight-map actions.

use macroquad_toolkit::ui::ScrollArea;

use crate::screens::{render_interplanetary_view, InterplanetaryAction};
use crate::{screens, state, Game, GamePhase};

impl Game {
    pub(super) fn update_interplanetary(&mut self) {
        self.advance_simulation();
        let stockpiles: [Option<f32>; state::PLANET_COUNT] =
            std::array::from_fn(|index| self.campaign.stockpile(index));
        let telemetry: [Option<state::WorldTelemetry>; state::PLANET_COUNT] =
            std::array::from_fn(|index| self.campaign.world_telemetry(index));
        let colonized = self.campaign.colonized_flags();
        let pads: [usize; state::PLANET_COUNT] = std::array::from_fn(|index| {
            self.campaign
                .planet(index)
                .map(|planet| planet.landing_pads_online())
                .unwrap_or(0)
        });
        let pads_built: [usize; state::PLANET_COUNT] = std::array::from_fn(|index| {
            self.campaign
                .planet(index)
                .map(|planet| planet.landing_pads_built())
                .unwrap_or(0)
        });
        let pad_positions: [Vec<crate::engine::GridPos>; state::PLANET_COUNT] =
            std::array::from_fn(|index| {
                self.campaign
                    .planet(index)
                    .map(|planet| planet.landing_pad_positions())
                    .unwrap_or_default()
            });
        let orders: [Option<state::ExportOrder>; state::PLANET_COUNT] =
            std::array::from_fn(|index| self.campaign.export_order_for(index));
        let drivers_online: [usize; state::PLANET_COUNT] = std::array::from_fn(|index| {
            self.campaign
                .planet(index)
                .map(|planet| planet.mass_drivers_online())
                .unwrap_or(0)
        });
        let drivers_built: [usize; state::PLANET_COUNT] = std::array::from_fn(|index| {
            self.campaign
                .planet(index)
                .map(|planet| planet.mass_drivers_built())
                .unwrap_or(0)
        });
        let pod_fractions: [f32; state::PLANET_COUNT] = std::array::from_fn(|index| {
            self.campaign
                .planet(index)
                .map(|planet| planet.pod_fraction())
                .unwrap_or(0.0)
        });
        let export_rates: [f32; state::PLANET_COUNT] = std::array::from_fn(|index| {
            self.campaign
                .planet(index)
                .map(|planet| planet.export_rate_per_minute())
                .unwrap_or(0.0)
        });
        let pending_pods: [usize; state::PLANET_COUNT] =
            std::array::from_fn(|index| self.campaign.pending_pod_count(index));
        let pod_caps: [usize; state::PLANET_COUNT] =
            std::array::from_fn(|index| self.campaign.pending_pod_cap(index));
        let overflow_pods: [usize; state::PLANET_COUNT] =
            std::array::from_fn(|index| self.campaign.overflow_pod_count(index));
        let current_world = self.campaign.current_index();
        let mass_driver_researched = self.has_mass_driver();
        let seed_ship_ready = self.campaign.current().seed_ship.is_ready_to_launch();
        let current_drivers_built = self.campaign.current().mass_drivers_built();
        let editing_world = self.shipping_edit_world;
        let mut shipping_scroll = std::mem::replace(&mut self.shipping_scroll, ScrollArea::new());
        let campaign = &self.campaign;
        let data = &self.data;
        let view = screens::MapView {
            data: data.data(),
            current_planet: current_world,
            mass_driver_researched,
            seed_ship_ready,
            colonized: &colonized,
            stockpiles: &stockpiles,
            telemetry: &telemetry,
            current_drivers_online: drivers_online[current_world],
            current_drivers_built,
            drivers_online: &drivers_online,
            drivers_built: &drivers_built,
            pads: &pads,
            pads_built: &pads_built,
            pad_positions: &pad_positions,
            pod_fractions: &pod_fractions,
            export_rates: &export_rates,
            shipments: campaign.shipments(),
            orders: &orders,
            pending_pods: &pending_pods,
            pod_caps: &pod_caps,
            overflow_pods: &overflow_pods,
            editing_world,
            launch_confirmation_target: self.launch_confirmation_target,
        };
        let action = render_interplanetary_view(
            &view,
            &mut shipping_scroll,
            self.interplanetary_mobile_panel,
        );
        self.shipping_scroll = shipping_scroll;
        match action {
            InterplanetaryAction::Close => {
                self.launch_confirmation_target = None;
                // Freight orders are campaign state, not just map controls.
                // Save on exit so a browser player does not lose a route
                // configured immediately before leaving this screen.
                self.save_campaign();
                self.phase = GamePhase::Playing;
            }
            InterplanetaryAction::CancelLaunch => {
                self.launch_confirmation_target = None;
                self.campaign
                    .current_mut()
                    .emit_audio(state::AudioEvent::UiBack);
            }
            InterplanetaryAction::SelectPlanet(index) => {
                self.launch_confirmation_target = None;
                if self.campaign.travel_to(index) {
                    self.shipping_edit_world = index;
                    self.shipping_scroll.set_offset(0.0);
                    self.sync_research_to_planet();
                    self.sync_building_unlocks();
                    self.save_campaign();
                    self.phase = GamePhase::Playing;
                }
            }
            InterplanetaryAction::RequestLaunch(index) => {
                if !self.campaign.is_colonized(index) {
                    self.launch_confirmation_target = Some(index);
                    self.campaign
                        .current_mut()
                        .emit_audio(state::AudioEvent::UiConfirm);
                }
            }
            InterplanetaryAction::LaunchSeedShip(index) => {
                self.launch_confirmation_target = None;
                let origin = self.campaign.current_index();
                if self.has_mass_driver() && self.campaign.launch_seed_ship(index) {
                    self.shipping_edit_world = index;
                    self.shipping_scroll.set_offset(0.0);
                    self.sync_research_to_planet();
                    self.sync_building_unlocks();
                    self.save_campaign();
                    // `launch_seed_ship` already armed the target world's
                    // arrival notice. Keep it through the cutscene so the
                    // first playable frame can identify where the swarm
                    // landed, including on the compact mobile header.
                    self.launch = Some(state::LaunchSequence::new_with_data(
                        self.data.clone(),
                        origin,
                        index,
                    ));
                    self.phase = GamePhase::Launch;
                }
            }
            InterplanetaryAction::CycleExportCargo => {
                self.save_shipping_edit(|campaign| campaign.cycle_export_cargo())
            }
            InterplanetaryAction::CycleExportTarget => {
                self.save_shipping_edit(|campaign| campaign.cycle_export_target())
            }
            InterplanetaryAction::CycleExportPad => {
                self.save_shipping_edit(|campaign| campaign.cycle_export_pad_for(editing_world))
            }
            InterplanetaryAction::CycleExportSchedule => self
                .save_shipping_edit(|campaign| campaign.cycle_export_schedule_for(current_world)),
            InterplanetaryAction::CycleExportPriority => self
                .save_shipping_edit(|campaign| campaign.cycle_export_priority_for(current_world)),
            InterplanetaryAction::CycleExportReserve => {
                self.save_shipping_edit(|campaign| campaign.cycle_export_reserve_for(current_world))
            }
            InterplanetaryAction::SelectOrderWorld(index) => {
                self.shipping_edit_world = index;
                self.shipping_scroll.set_offset(0.0);
            }
            InterplanetaryAction::CycleRemoteExportCargo => {
                self.save_shipping_edit(|campaign| campaign.cycle_export_cargo_for(editing_world))
            }
            InterplanetaryAction::CycleRemoteExportTarget => {
                self.save_shipping_edit(|campaign| campaign.cycle_export_target_for(editing_world))
            }
            InterplanetaryAction::CycleRemoteExportSchedule => self
                .save_shipping_edit(|campaign| campaign.cycle_export_schedule_for(editing_world)),
            InterplanetaryAction::CycleRemoteExportPriority => self
                .save_shipping_edit(|campaign| campaign.cycle_export_priority_for(editing_world)),
            InterplanetaryAction::CycleRemoteExportReserve => {
                self.save_shipping_edit(|campaign| campaign.cycle_export_reserve_for(editing_world))
            }
            InterplanetaryAction::ToggleMobileWorlds => {
                self.launch_confirmation_target = None;
                self.interplanetary_mobile_panel = state::InterplanetaryMobilePanel::Worlds;
                self.shipping_scroll.set_offset(0.0);
            }
            InterplanetaryAction::ToggleMobileShipping => {
                self.launch_confirmation_target = None;
                self.interplanetary_mobile_panel = state::InterplanetaryMobilePanel::Shipping;
                self.shipping_scroll.set_offset(0.0);
            }
            InterplanetaryAction::None => {}
        }
    }

    fn save_shipping_edit(&mut self, edit: impl FnOnce(&mut state::Campaign)) {
        edit(&mut self.campaign);
        self.save_campaign();
    }
}
