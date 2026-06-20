use rand::RngExt;
use std::sync::Arc;
use steel_macros::block_behavior;
use steel_registry::blocks::block_state_ext::BlockStateExt;
use steel_registry::blocks::properties::{BlockStateProperties, BoolProperty, EnumProperty, Tilt};
use steel_registry::sound_event::SoundEventRef;
use steel_registry::sound_events::{BLOCK_BIG_DRIPLEAF_TILT_DOWN, BLOCK_BIG_DRIPLEAF_TILT_UP};
use steel_registry::vanilla_block_tags::BlockTag;
use steel_registry::vanilla_blocks;
use steel_utils::types::UpdateFlags;
use steel_utils::{BlockPos, BlockStateId};

use crate::behavior::block::BlockBehavior;
use crate::behavior::context::BlockPlaceContext;
use crate::entity::{Entity, InsideBlockEffectCollector};
use crate::world::tick_scheduler::TickPriority;
use crate::world::{LevelReader, World};

use super::BlockRef;

const TILT: EnumProperty<Tilt> = BlockStateProperties::TILT;
const WATERLOGGED: BoolProperty = BlockStateProperties::WATERLOGGED;

/// Vanilla `BigDripleafBlock` survival.
///
/// Survives if the block below is big dripleaf (self), big dripleaf stem, or
/// in the `SUPPORTS_BIG_DRIPLEAF` tag.
// TODO: Implement tilt-on-stand, projectile tilt, bonemeal stem growth.
#[block_behavior]
pub struct BigDripleafBlock {
    block: BlockRef,
}

impl BigDripleafBlock {
    /// Creates a new big dripleaf block behavior.
    #[must_use]
    pub const fn new(block: BlockRef) -> Self {
        Self { block }
    }
    fn can_entity_tilt(pos: &BlockPos, entity: &dyn Entity) -> bool {
        entity.on_ground() && entity.position().y > pos.y() as f64 + 0.6875_f64
    }
    fn set_tilt_and_schedule_tick(
        &self,
        state_id: BlockStateId,
        world: &Arc<World>,
        pos: &BlockPos,
        tilt: Tilt,
        sound_wrapper: Option<SoundEventRef>,
    ) {
        Self::set_tilt(state_id, world, pos, tilt.clone());
        if let Some(tilt_sound) = sound_wrapper {
            Self::play_tilt_sound(world, pos, tilt_sound);
        }
        let tick_delay = match tilt {
            Tilt::None => None,
            Tilt::Unstable | Tilt::Partial => Some(10),
            Tilt::Full => Some(100),
        };
        if let Some(tick_delay) = tick_delay {
            world.schedule_block_tick(*pos, self.block, tick_delay, TickPriority::Normal);
        }
    }
    fn set_tilt(state_id: BlockStateId, world: &Arc<World>, pos: &BlockPos, new_tilt: Tilt) {
        world.set_block(
            *pos,
            state_id.set_value(&TILT, new_tilt),
            UpdateFlags::UPDATE_ALL,
        );
    }
    fn play_tilt_sound(world: &Arc<World>, pos: &BlockPos, tilt_sound: SoundEventRef) {
        let pitch = rand::rng().random_range(0.8f32..1.2f32);
        world.play_block_sound(tilt_sound, *pos, 1f32, pitch, None);
    }
    fn reset_tilt(state_id: BlockStateId, world: &Arc<World>, pos: &BlockPos) {
        Self::set_tilt(state_id, world, pos, Tilt::None);
        let tilt = state_id.get_value(&TILT);

        if tilt != Tilt::None {
            Self::play_tilt_sound(world, pos, &BLOCK_BIG_DRIPLEAF_TILT_UP);
        }
    }
}

impl BlockBehavior for BigDripleafBlock {
    fn can_survive(&self, _state: BlockStateId, world: &dyn LevelReader, pos: BlockPos) -> bool {
        let below = world.get_block_state(pos.below());
        let below_block = below.get_block();
        below_block == self.block
            || below_block == &vanilla_blocks::BIG_DRIPLEAF_STEM
            || below_block.has_tag(&BlockTag::SUPPORTS_BIG_DRIPLEAF)
    }

    fn get_state_for_placement(&self, context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        let state = self.block.default_state();
        self.can_survive(state, context.world, context.relative_pos)
            .then_some(state.set_value(&WATERLOGGED, context.is_water_source()))
    }
    fn entity_inside(
        &self,
        state: BlockStateId,
        world: &Arc<World>,
        pos: BlockPos,
        entity: &dyn Entity,
        _effect_collector: &mut InsideBlockEffectCollector,
        _is_precise: bool,
    ) {
        let tilt = state.get_value(&TILT);
        //TODO: also check !level.hasNeighborSignal(pos)) once steel implements redstone
        if tilt == Tilt::None && BigDripleafBlock::can_entity_tilt(&pos, entity) {
            Self::set_tilt_and_schedule_tick(&self, state, world, &pos, Tilt::Unstable, None);
        }
    }
    fn tick(&self, state: BlockStateId, world: &Arc<World>, pos: BlockPos) {
        //if block_receives_redstone_power(world, pos) {
        //    reset_tilt(state.id, world, pos);
        //} else {
        let tilt = state.get_value(&TILT);

        if tilt == Tilt::Unstable {
            Self::set_tilt_and_schedule_tick(
                &self,
                state,
                world,
                &pos,
                Tilt::Partial,
                Some(&BLOCK_BIG_DRIPLEAF_TILT_DOWN),
            );
        } else if tilt == Tilt::Partial {
            Self::set_tilt_and_schedule_tick(
                &self,
                state,
                world,
                &pos,
                Tilt::Full,
                Some(&BLOCK_BIG_DRIPLEAF_TILT_DOWN),
            );
        } else if tilt == Tilt::Full {
            Self::reset_tilt(state, world, &pos);
        }
        //}
    }
}
