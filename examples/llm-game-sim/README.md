# NOTE: This is not supported yet.

llm-game-sim/
├── loqi.toml
├── src/
│   ├── main.loq                  # Entry point + loop
│   ├── game_loop.loq             # Tick-based orchestrator
│   ├── state/
│   │   ├── schemas/
│   │   │   ├── player.schema.loq
│   │   │   ├── inventory.schema.loq
│   │   │   ├── world.schema.loq
│   │   │   ├── event.schema.loq
│   │   │   └── combat.schema.loq
│   │   └── models/
│   │       └── memory_ai.model.loq
│   ├── systems/
│   │   ├── dialogue/
│   │   │   ├── prompts/
│   │   │   │   ├── npc_dialogue.loq
│   │   │   │   └── branching_dialogue.loq
│   │   │   └── schemas/dialogue.schema.loq
│   │   ├── crafting/
│   │   │   ├── prompts/crafting_action.loq
│   │   │   └── schemas/recipe.schema.loq
│   │   ├── combat/
│   │   │   ├── prompts/combat_action.loq
│   │   │   └── schemas/combat_result.schema.loq
│   │   └── narration/
│   │       ├── prompts/world_tick.loq
│   │       └── schemas/narration.schema.loq
│   ├── assets/
│   │   ├── prompts/
│   │   │   ├── character_design.loq
│   │   │   └── environment_design.loq
│   │   └── schemas/asset.schema.loq
│   └── tools/
│       ├── state_manager.loq
│       ├── combat_tools.loq
│       └── crafting_tools.loq
└── outputs/
    ├── audio/
    ├── images/
    ├── models/
    └── state/
