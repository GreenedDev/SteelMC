use std::ops::Add;
use std::sync::Arc;

use glam::DVec3;
use rand::{Rng, RngExt};
use steel_macros::block_behavior;
use steel_registry::blocks::block_state_ext::BlockStateExt;
use steel_registry::entity_data::ParticleData;
use steel_registry::entity_type::EntityType;
use steel_registry::mob_effect::MobEffect;
use steel_registry::particle_type::TrailParticleOption;
use steel_registry::sound_event::SoundEvent;
use steel_registry::vanilla_block_tags::BlockTag;
use steel_registry::{
    sound_events, vanilla_blocks, vanilla_entities, vanilla_entity_type_tags, vanilla_game_events,
    vanilla_mob_effects, vanilla_particle_types,
};
use steel_utils::types::{Difficulty, UpdateFlags};
use steel_utils::{BlockPos, BlockStateId, RgbColor};

use crate::behavior::block::BlockBehavior;
use crate::behavior::context::BlockPlaceContext;
use crate::entity::{Entity, InsideBlockEffectCollector};
use crate::world::game_event::GameEventContext;
use crate::world::{LevelReader, World};

use super::{BlockRef, default_surviving_state, survives_on_tag};

/// Vanilla `EyeblossomBlock` survival and ticking shape.
// TODO: Implement eyeblossom day/night transforms, sounds, particles, and bee effects
// once Steel has environment attributes and particle dispatch.
#[block_behavior]
pub struct EyeblossomBlock {
    block: BlockRef,
    #[json_arg(r#enum = "EyeblossomType", json = "type")]
    eyeblossom_type: EyeblossomType,
}

impl EyeblossomBlock {
    /// Creates a new eyeblossom behavior.
    #[must_use]
    pub const fn new(block: BlockRef, eyeblossom_type: EyeblossomType) -> Self {
        Self {
            block,
            eyeblossom_type,
        }
    }
    fn try_changing_state(
        &self,
        state: BlockStateId,
        level: &Arc<World>,
        pos: BlockPos,
        random: &mut dyn Rng,
    ) -> bool {
        let should_be_open: bool = true; //TODO: should_be_open = level.environmental_attributes().get_value(EnvironmentAttributes::EYEBLOSSOM_OPEN, pos).to_boolean(self.ty.open());

        if should_be_open == self.eyeblossom_type.open() {
            return false;
        }

        let new_type = self.eyeblossom_type.transform();
        level.set_block(pos, new_type.state(), UpdateFlags::UPDATE_ALL);
        level.game_event(
            &vanilla_game_events::BLOCK_CHANGE,
            pos,
            &GameEventContext::new(None, Some(state)),
        );
        new_type.spawn_transform_particle(level, &pos, random);

        let from = pos.offset(-3, -2, -3);
        let to = pos.offset(3, 2, 3);
        for nearby in BlockPos::between_closed(from, to) {
            let nearby_state = level.get_block_state(nearby);
            if nearby_state == state {
                let distance = (pos.0.distance_squared(nearby.0) as f64).sqrt();
                let delay = random.random_range((distance * 5.0) as i32..(distance * 10.0) as i32);
                level.schedule_block_tick_default(nearby, state.get_block(), delay);
            }
        }

        true
    }
}

impl BlockBehavior for EyeblossomBlock {
    fn can_survive(&self, _state: BlockStateId, world: &dyn LevelReader, pos: BlockPos) -> bool {
        survives_on_tag(world, pos, &BlockTag::SUPPORTS_VEGETATION)
    }

    fn get_state_for_placement(&self, context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        default_surviving_state(self.block, self, context)
    }

    fn random_tick(&self, state: BlockStateId, world: &Arc<World>, pos: BlockPos) {
        let sound = self.eyeblossom_type.transform().long_switch_sound();
        if Self::try_changing_state(self, state, world, pos, &mut rand::rng()) {
            world.play_block_sound(sound, pos, 1.0, 1.0, None);
        }
    }

    fn tick(&self, _state: BlockStateId, _world: &Arc<World>, _pos: BlockPos) {
        let _ = self.eyeblossom_type;
    }

    fn entity_inside(
        &self,
        _state: BlockStateId,
        world: &Arc<World>,
        _pos: BlockPos,
        entity: &dyn Entity,
        _effect_collector: &mut InsideBlockEffectCollector,
        _is_precise: bool,
    ) {
        if world.difficulty() != Difficulty::Peaceful
            && entity.entity_type() == &vanilla_entities::BEE
        { //TODO: && Bee.attractsBees(state) && !bee.hasEffect(MobEffects.POISON)) {
            //TODO: entity.add_effect(self.get_bee_interaction_effect())
        }
    }
}

#[derive(Clone, Copy)]
/// Vanilla open/closed eyeblossom type from `classes.json`.
pub enum EyeblossomType {
    /// Emits open-eyeblossom effects and transforms closed at daytime.
    Open,
    /// Emits closed-eyeblossom effects and transforms open at nighttime.
    Closed,
}

impl EyeblossomType {
    const fn open(self) -> bool {
        match self {
            Self::Open => true,
            Self::Closed => false,
        }
    }

    const fn effect(self) -> &'static MobEffect {
        match self {
            Self::Open => vanilla_mob_effects::BLINDNESS,
            Self::Closed => vanilla_mob_effects::NAUSEA,
        }
    }

    const fn effect_duration(self) -> f32 {
        match self {
            Self::Open => 11.0,
            Self::Closed => 7.0,
        }
    }

    const fn long_switch_sound(self) -> &'static SoundEvent {
        match self {
            Self::Open => &sound_events::BLOCK_EYEBLOSSOM_OPEN_LONG,
            Self::Closed => &sound_events::BLOCK_EYEBLOSSOM_CLOSE_LONG,
        }
    }

    const fn short_switch_sound(self) -> &'static SoundEvent {
        match self {
            Self::Open => &sound_events::BLOCK_EYEBLOSSOM_OPEN,
            Self::Closed => &sound_events::BLOCK_EYEBLOSSOM_CLOSE,
        }
    }

    const fn particle_color(self) -> i32 {
        match self {
            Self::Open => 16_545_810,
            Self::Closed => 6_250_335,
        }
    }

    fn block(self) -> BlockRef {
        match self {
            Self::Open => &vanilla_blocks::OPEN_EYEBLOSSOM,
            Self::Closed => &vanilla_blocks::CLOSED_EYEBLOSSOM,
        }
    }

    fn state(self) -> BlockStateId {
        self.block().default_state()
    }

    const fn transform(self) -> Self {
        match self {
            Self::Open => Self::Closed,
            Self::Closed => Self::Open,
        }
    }

    const fn emit_sounds(self) -> bool {
        matches!(self, Self::Open)
    }

    const fn from_boolean(open: bool) -> Self {
        if open { Self::Open } else { Self::Closed }
    }

    fn spawn_transform_particle(self, level: &Arc<World>, pos: &BlockPos, random: &mut dyn Rng) {
        let mut start = DVec3::from(pos.0);
        start = start.map(|n| n + 0.5);

        let lifetime = 0.5 + random.random::<f64>();

        let velocity = DVec3::new(
            random.random::<f64>() - 0.5,
            random.random::<f64>() + 1.0,
            random.random::<f64>() - 0.5,
        );

        let target = start.add(velocity * lifetime);

        let particle = TrailParticleOption::new(
            target,
            RgbColor::new(self.particle_color()),
            (20.0 * lifetime) as i32,
        );

        level.send_particles(
            ParticleData::new(&vanilla_particle_types::BLOCK, particle),
            start,
            1,
            DVec3::splat(0.0),
            0.0,
        );
    }
}

#[cfg(test)]
mod tests {
    use steel_registry::{init_vanilla_registry, vanilla_blocks};
    use steel_utils::BlockPos;

    use crate::test_support::TestLevel;

    use super::*;

    fn level_with_support(support: BlockRef) -> TestLevel {
        TestLevel::default().with_block(BlockPos::new(0, 63, 0), support.default_state())
    }

    #[test]
    fn eyeblossom_requires_vegetation_support() {
        init_vanilla_registry();
        let behavior =
            EyeblossomBlock::new(&vanilla_blocks::CLOSED_EYEBLOSSOM, EyeblossomType::Closed);
        let pos = BlockPos::new(0, 64, 0);
        let state = vanilla_blocks::CLOSED_EYEBLOSSOM.default_state();

        assert!(behavior.can_survive(state, &level_with_support(&vanilla_blocks::DIRT), pos));
        assert!(!behavior.can_survive(state, &level_with_support(&vanilla_blocks::AIR), pos));
    }
}
