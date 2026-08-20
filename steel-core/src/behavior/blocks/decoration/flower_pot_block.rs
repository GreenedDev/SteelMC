//! Torch block implementations.
//!
//! Torches come in two forms:
//! - Standing torches (TorchBlock): placed on top of blocks
//! - Wall torches (WallTorchBlock): placed on the side of blocks
//!
//! Both break when their supporting block is removed.

use std::sync::Arc;

use steel_macros::block_behavior;
use steel_registry::blocks::BlockRef;
use steel_registry::blocks::block_state_ext::BlockStateExt;
use steel_registry::blocks::properties::{BlockStateProperties, Direction, EnumProperty};
use steel_registry::blocks::shapes::SupportType;
use steel_registry::game_events::GameEvent;
use steel_registry::item_stack::ItemStack;
use steel_registry::items::item::BlockHitResult;
use steel_registry::vanilla_blocks;
use steel_registry::{REGISTRY, vanilla_game_events};
use steel_utils::types::{InteractionHand, UpdateFlags};
use steel_utils::{BlockPos, BlockStateId};

use crate::behavior::block::BlockBehavior;
use crate::behavior::blocks::EyeblossomBlock;
use crate::behavior::context::BlockPlaceContext;
use crate::behavior::{InteractionResult, InventoryAccess};
use crate::entity::ai::path::PathComputationType;
use crate::inventory::container::Container;
use crate::player::Player;
use crate::world::game_event::GameEventContext;
use crate::world::{LevelReader, ScheduledTickAccess, World};

/// Behavior for standing torch blocks (torch, `soul_torch`, `copper_torch`).
///
/// Standing torches are placed on top of blocks and require center support
/// from the block below to survive.
#[block_behavior]
pub struct FlowerPotBlock {
    block: BlockRef,
    #[json_arg(vanilla_blocks, json = "potted")]
    potted: BlockRef,
}

impl FlowerPotBlock {
    /// Creates a new standing torch block behavior.
    #[must_use]
    pub const fn new(block: BlockRef, potted: BlockRef) -> Self {
        Self { block, potted }
    }
    fn is_empty(&self) -> bool {
        self.potted == &vanilla_blocks::AIR
    }
    fn opposite(state: BlockStateId) -> BlockStateId {
        if state.get_block() == &vanilla_blocks::POTTED_OPEN_EYEBLOSSOM {
            return vanilla_blocks::POTTED_CLOSED_EYEBLOSSOM.default_state();
        }
        if state.get_block() == &vanilla_blocks::POTTED_CLOSED_EYEBLOSSOM {
            return vanilla_blocks::POTTED_OPEN_EYEBLOSSOM.default_state();
        }
        state
    }
}

impl BlockBehavior for FlowerPotBlock {
    fn use_without_item(
        &self,
        state: BlockStateId,
        world: &Arc<World>,
        pos: BlockPos,
        player: &Player,
        hit_result: &BlockHitResult,
        inv: &mut InventoryAccess,
    ) -> InteractionResult {
        if Self::is_empty(&self) {
            return InteractionResult::Consume;
        }

        let plant = ItemStack::new(REGISTRY.items.by_block(self.potted));
        if !player.inventory.lock().add(&mut plant) {
            player.drop_item(plant, false, false);
        }
        world.set_block(
            pos,
            vanilla_blocks::FLOWER_POT.default_state(),
            UpdateFlags::UPDATE_ALL,
        );
        world.game_event(
            &vanilla_game_events::BLOCK_CHANGE,
            pos,
            &GameEventContext::new(Some(player), None),
        );
        InteractionResult::Success
    }
    fn get_clone_item_stack(
        &self,
        block: BlockRef,
        _state: BlockStateId,
        _include_data: bool,
    ) -> Option<ItemStack> {
        if Self::is_empty(&self) {
            return Some(ItemStack::new(REGISTRY.items.by_block(block)));
        }
        Some(ItemStack::new(REGISTRY.items.by_block(self.potted)))
    }
    fn update_shape(
        &self,
        state: BlockStateId,
        world: &dyn ScheduledTickAccess,
        pos: BlockPos,
        direction: Direction,
        _neighbor_pos: BlockPos,
        _neighbor_state: BlockStateId,
    ) -> BlockStateId {
        if direction == Direction::Down && !self.can_survive(state, world, pos) {
            return REGISTRY.blocks.get_default_state_id(&vanilla_blocks::AIR);
        }
        state
    }
    fn is_pathfindable(
        &self,
        _state: BlockStateId,
        _computation_type: PathComputationType,
    ) -> bool {
        false
    }
    fn random_tick(&self, state: BlockStateId, world: &Arc<World>, pos: BlockPos) {
        if self.block.randomly_ticking {
            let is_open = self.potted == &vanilla_blocks::OPEN_EYEBLOSSOM;
            let should_be_open = true; //TODO: shouldBeOpen = ((TriState)level.environmentAttributes().getValue(EnvironmentAttributes.EYEBLOSSOM_OPEN, pos)).toBoolean(isOpen);
            if !is_open != should_be_open {
                world.set_block(pos, Self::opposite(state), UpdateFlags::UPDATE_ALL);
                //let new_type =
            }
        }
    }
}
