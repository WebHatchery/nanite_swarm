//! Producer buffers, partial-load policy, and drone dispatch.

use crate::engine::{BuildingType, DroneState, GridPos, ResourceType};

use super::PlanetState;

/// How much a producer may stockpile on its pad while its drones are away, as
/// a multiple of a drone load. Past this it simply stops producing: a building
/// that outruns its logistics is the pressure, not free storage.
const PAD_LOADS: f32 = 3.0;

impl PlanetState {
    /// Whether a part-full pad is worth a trip.
    ///
    /// Only on a run long enough that the walk dominates, and only once the pad
    /// holds a worthwhile share of a load — otherwise a crew would trickle out
    /// single grains of ore.
    fn worth_a_partial_load(&self, carried: f32, load: f32, route_len: usize) -> bool {
        if carried <= 0.0 || load <= 0.0 {
            return false;
        }
        // Keep the old data field as an explicit "never partial" switch for
        // compatibility with balance fixtures; ordinary gameplay uses the
        // derived timing below and does not compare against a tile threshold.
        if self.config.buildings.partial_load_min_route >= 1_000.0 {
            return false;
        }
        let drill_rate = self.drill_output_rate().max(0.001);
        let fill_time = load / drill_rate;
        // Include the pickup/drop-off hop in the route estimate. The path
        // stores network tiles, while a real trip also leaves the producer
        // and arrives at the consumer.
        let trip_time = (route_len as f32 + 1.0) / self.drones.drone_speed.max(0.001) * 2.0;
        if trip_time <= fill_time * 0.5 {
            return false;
        }
        let lookahead = self.config.logistics.dispatch_lookahead.max(0.0);
        // A short walk should wait out the remaining fill; a long walk should
        // leave as soon as the waiting cost is smaller than another round
        // trip. The fill-time term and the configured dispatch look-ahead keep
        // this derived from the live route and production rate rather than a
        // tile-count constant.
        let waiting_cost = fill_time + lookahead;
        let required_share = (waiting_cost / (trip_time + waiting_cost)).clamp(0.1, 1.0);
        carried / load >= required_share
    }

    /// Cut ore into each drill's buffer.
    ///
    /// Everything a drill produces goes on its pad; getting it anywhere is
    /// [`Self::dispatch_producers`]'s problem, the same as for a Smelter.
    pub(super) fn update_drills(&mut self, delta_time: f32) {
        let rate = self.drill_output_rate();
        let dust_response = self.resolved_dust_response();
        let crew = self.drone_crew_size();
        let ceiling = self
            .config
            .logistics
            .hopper_depth
            .max(self.drones.drone_capacity * PAD_LOADS * crew as f32);
        let depletion = self.config.ore.depletion_per_unit.max(0.0);

        for drill_pos in self.grid.find_buildings(BuildingType::Drill) {
            let feature_multiplier = self.feature_ore_multiplier_at(drill_pos).unwrap_or(1.0);
            let Some(tile) = self.grid.get(drill_pos) else {
                continue;
            };
            let Some(building) = tile.building.as_ref() else {
                continue;
            };
            if !building.powered || building.is_dust_stalled_with(&dust_response) {
                continue;
            }
            // What the ground under this particular drill is worth. Placement
            // is a spatial decision, not only a question of run length.
            let richness = tile.ore_richness;
            let efficiency = building.dust_efficiency_with(&dust_response);
            let cut = rate * richness * feature_multiplier * efficiency * delta_time;

            let buffer = self
                .output_buffers
                .entry((drill_pos.x, drill_pos.y))
                .or_insert(0.0);
            let before = *buffer;
            *buffer = (before + cut).min(ceiling);

            // Only the bonus is spent: a deposit is cut down towards ordinary
            // ground and stops there (gdd.md §3). What the pad could not hold
            // was never cut, so it is not charged for either.
            if richness > 1.0 && depletion > 0.0 {
                let taken = *buffer - before;
                if let Some(tile) = self.grid.get_mut(drill_pos) {
                    tile.ore_richness = (richness - taken * depletion).max(1.0);
                }
            }
        }
    }

    /// Give every producer a crew, and send a drone whenever a full load is
    /// waiting on its pad.
    ///
    /// A drill and a Smelter are the same problem here: something has piled up
    /// somewhere and has to be carried to whatever wants it. Only the resource
    /// and therefore the destination differ.
    pub(super) fn dispatch_producers(&mut self, core: GridPos) {
        let load = self.drones.drone_capacity;
        let crew = self.drone_crew_size();

        for (producer, resource) in self.producers() {
            // Research can grow a crew; the extra drones turn up at the next
            // cycle rather than needing the building rebuilt.
            while self.drones.drones_at(producer).len() < crew {
                self.drones.spawn_drone(producer);
            }

            let waiting = self
                .output_buffers
                .get(&(producer.x, producer.y))
                .copied()
                .unwrap_or(0.0);
            if waiting <= 0.0 {
                continue;
            }

            // Ore goes to whatever wants it and is nearest on the network, and
            // to the Core only when nothing does. Refined output has no
            // consumer yet, so it always goes home to the Core.
            // Everything looks for something that wants it and is nearest on
            // the network, and goes home to the Core only when nothing does.
            let delivery = self.delivery_for(producer, core, resource);
            let Some((destination, route)) = delivery else {
                // Powered but no pipe anywhere: it piles up on the pad instead
                // of teleporting into the pool.
                continue;
            };

            // On a long run a drone that waits for a full load stands still
            // while ore piles up behind it, which is why a second drone used to
            // be worth nothing except on the longest runs. On a short run
            // waiting costs almost nothing, because it comes straight back.
            let carried = waiting.min(load);
            if carried < load && !self.worth_a_partial_load(carried, load, route.len()) {
                continue;
            }

            let idle_drone = self
                .drones
                .drones()
                .iter()
                .find(|d| d.home == producer && d.state == DroneState::Idle)
                .map(|d| d.id);

            let Some(drone_id) = idle_drone else {
                if let Some(tile) = route.first() {
                    let queue = self.drone_queues.entry((tile.x, tile.y)).or_default();
                    if !queue.contains(&self.drones.drones_at(producer).first().map_or(0, |d| d.id))
                        && queue.len() < self.config.logistics.max_queue_depth
                    {
                        if let Some(drone) = self.drones.drones_at(producer).first() {
                            queue.push(drone.id);
                        }
                    }
                }
                continue;
            };
            if let Some(buffer) = self.output_buffers.get_mut(&(producer.x, producer.y)) {
                *buffer -= carried;
            }
            if let Some(drone) = self.drones.get_drone_mut(drone_id) {
                drone.dispatch(destination, route, carried, resource);
                for tile in drone
                    .path
                    .iter()
                    .take(self.config.logistics.dispatch_lookahead.max(1.0) as usize)
                {
                    self.route_reservations.insert(
                        (tile.x, tile.y),
                        self.config.logistics.reservation_seconds.max(0.0),
                    );
                }
            }
            if let Some(tile) = self
                .drones
                .get_drone_mut(drone_id)
                .and_then(|drone| drone.path.first().copied())
            {
                if let Some(queue) = self.drone_queues.get_mut(&(tile.x, tile.y)) {
                    queue.retain(|queued| *queued != drone_id);
                }
            }
        }
    }

    /// Every powered building that piles something up for collection, and what
    /// that something is.
    fn producers(&self) -> Vec<(GridPos, ResourceType)> {
        let mut producers: Vec<(GridPos, ResourceType)> = self
            .powered_positions(BuildingType::Drill)
            .into_iter()
            .map(|pos| (pos, ResourceType::Minerals))
            .collect();

        // Anything whose recipe makes something a drone can carry is a
        // producer too, whatever that something turns out to be.
        for def in &self.data().buildings {
            let Some(kind) = BuildingType::from_id(&def.id) else {
                continue;
            };
            for (id, rate) in &def.recipe.outputs {
                let Some(resource) = ResourceType::from_id(id) else {
                    continue;
                };
                if *rate <= 0.0 || !resource.is_physical() {
                    continue;
                }
                producers.extend(
                    self.powered_positions(kind)
                        .into_iter()
                        .map(|pos| (pos, resource)),
                );
            }
        }

        // A Landing Pad with a pod on it is a producer of whatever that pod
        // held. Nothing else about it is special: the cargo sits there until a
        // drone walks it somewhere, so a pad with no pipe to it is a pile.
        for pos in self.powered_positions(BuildingType::LandingPad) {
            let key = (pos.x, pos.y);
            if self.output_buffers.get(&key).copied().unwrap_or(0.0) <= 0.0 {
                continue;
            }
            if let Some(cargo) = self.pad_cargo.get(&key) {
                producers.push((pos, *cargo));
            }
        }

        producers
    }
}
