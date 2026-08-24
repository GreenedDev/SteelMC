use std::sync::{Arc, LazyLock};

use rand::{Rng, RngExt};
use steel_macros::block_behavior;
use steel_registry::REGISTRY;
use steel_registry::blocks::block_state_ext::BlockStateExt;
use steel_registry::feature::ConfiguredFeature;
use steel_registry::feature::ConfiguredFeatureKind;
use steel_utils::Direction;
use steel_utils::random::worldgen_random::WorldgenRandom;
use steel_utils::{BlockPos, BlockStateId, Identifier};

use crate::behavior::blocks::vegetation::vegetation_block::survival_update_shape;
use crate::behavior::context::BlockPlaceContext;
use crate::behavior::{block::BlockBehavior, blocks::vegetation::bonemealable::Bonemealable};
use crate::world::ScheduledTickAccess;
use crate::world::{LevelReader, World};
use crate::worldgen::feature::FeatureDecorationRunner;

use super::{BlockRef, default_surviving_state, survives_on_tag};

/// Vanilla `NetherFungusBlock` survival.
#[block_behavior]
pub struct NetherFungusBlock {
    block: BlockRef,
    #[json_arg(vanilla_configured_features, json = "feature")]
    feature: &'static LazyLock<ConfiguredFeature>,
    #[json_arg(vanilla_blocks)]
    required_block: BlockRef,
    #[json_arg(vanilla_block_tags)]
    support_blocks: Identifier,
}

impl NetherFungusBlock {
    /// Creates a new nether fungus block behavior.
    #[must_use]
    pub const fn new(
        block: BlockRef,
        feature: &'static LazyLock<ConfiguredFeature>,
        required_block: BlockRef,
        support_blocks: Identifier,
    ) -> Self {
        Self {
            block,
            feature,
            required_block,
            support_blocks,
        }
    }
}

impl BlockBehavior for NetherFungusBlock {
    fn can_survive(&self, _state: BlockStateId, world: &dyn LevelReader, pos: BlockPos) -> bool {
        survives_on_tag(world, pos, &self.support_blocks)
    }

    fn update_shape(
        &self,
        state: BlockStateId,
        world: &dyn ScheduledTickAccess,
        pos: BlockPos,
        _direction: Direction,
        _neighbor_pos: BlockPos,
        _neighbor_state: BlockStateId,
    ) -> BlockStateId {
        survival_update_shape(self, state, world, pos)
    }

    fn get_state_for_placement(&self, context: &BlockPlaceContext<'_>) -> Option<BlockStateId> {
        default_surviving_state(self.block, self, context)
    }

    fn as_bonemealable(&self) -> Option<&dyn Bonemealable> {
        Some(self)
    }
}

impl Bonemealable for NetherFungusBlock {
    fn is_valid_bonemeal_target(
        &self,
        _state: BlockStateId,
        world: &dyn LevelReader,
        pos: BlockPos,
    ) -> bool {
        let below_state = world.get_block_state(pos.below());
        below_state.get_block() == self.required_block
            && !world.is_outside_build_height(pos.above().y())
    }

    fn is_bonemeal_success(
        &self,
        _state: BlockStateId,
        _world: &Arc<World>,
        rng: &mut dyn Rng,
        _pos: BlockPos,
    ) -> bool {
        rng.random::<f64>() < 0.4
    }

    fn perform_bonemeal(
        &self,
        _state: BlockStateId,
        world: &Arc<World>,
        rng: &mut dyn Rng,
        pos: BlockPos,
    ) {
        let configured_feature = &**self.feature;
        let mut worldgen_random = WorldgenRandom::from_seed(rng.random());

        if let ConfiguredFeatureKind::HugeFungus(config) = &configured_feature.kind {
            FeatureDecorationRunner::place_huge_fungus_feature(
                world,
                &REGISTRY,
                &mut worldgen_random,
                config,
                pos,
            );
        }
    }
}
