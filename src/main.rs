use minigene::*;
use std::collections::HashMap;
use std::ops::Deref;

add_wasm_support!();

const PLAY_WIDTH: u32 = 81;
const PLAY_HEIGHT: u32 = 50;
const SCREEN_WIDTH: u32 = 100;
const SCREEN_HEIGHT: u32 = 50;
const CREEP_SPAWN_TICKS: u32 = 50;
const CREEP_ATTACK_RADIUS: f32 = 2.1;
const LEADER_ATTACK_RADIUS: f32 = 2.1;
const AOE_RADIUS: f32 = 4.0;
const AOE_DAMAGE: f64 = 100.0;
const TOWER_RANGE: f32 = 5.0;
const TOWER_PROJECTILE_EXPLOSION_RADIUS: f32 = 2.1;
const SPRITESHEET_PATH: &str = "./assets/tilemap/colored_tilemap_packed.png";

const MAP: &[&str] = &[
    "####################################000000000####################################",
    "####################################000000000####################################",
    "####################################000000000####################################",
    "#########################0000000000000000000000000000000#########################",
    "#########################0000000000000000000000000000000#########################",
    "#########################0000000000000000000000000000000#########################",
    "#########################0000000000000000000000000000000#########################",
    "##################000000000000000000000000000000000000000000000##################",
    "##################000000000000000000000000000000000000000000000##################",
    "##################000000000000000000000000000000000000000000000##################",
    "##################00000###############00000###############00000##################",
    "##################00000###############00000###############00000##################",
    "##################00000###############00000###############00000##################",
    "##################00000###############00000###############00000##################",
    "##################00000###############00000###############00000##################",
    "##################00000###############00000###############00000##################",
    "##################00000###############00000###############00000##################",
    "##################00000###############00000###############00000##################",
    "##################00000###############00000###############00000##################",
    "##################00000###############00000###############00000##################",
    "##################00000###############00000###############00000##################",
    "##################00000###############00000###############00000##################",
    "##################00000###############00000###############00000##################",
    "##################00000###############00000###############00000##################",
    "##################00000###############00000###############00000##################",
    "##################00000###############00000###############00000##################",
    "##################00000###############00000###############00000##################",
    "##################00000###############00000###############00000##################",
    "##################00000###############00000###############00000##################",
    "##################00000###############00000###############00000##################",
    "##################00000###############00000###############00000##################",
    "##################00000###############00000###############00000##################",
    "##################00000###############00000###############00000##################",
    "##################00000###############00000###############00000##################",
    "##################00000###############00000###############00000##################",
    "##################00000###############00000###############00000##################",
    "##################00000###############00000###############00000##################",
    "##################00000###############00000###############00000##################",
    "##################00000###############00000###############00000##################",
    "##################00000###############00000###############00000##################",
    "##################000000000000000000000000000000000000000000000##################",
    "##################000000000000000000000000000000000000000000000##################",
    "##################000000000000000000000000000000000000000000000##################",
    "#########################0000000000000000000000000000000#########################",
    "#########################0000000000000000000000000000000#########################",
    "#########################0000000000000000000000000000000#########################",
    "#########################0000000000000000000000000000000#########################",
    "####################################000000000####################################",
    "####################################000000000####################################",
    "####################################000000000####################################",
];


mod components;
mod events;
mod ids;
mod render_map;
mod resources;
mod states;
mod systems;
mod utils;
pub use self::components::*;
pub use self::events::*;
pub use self::ids::*;
pub use self::render_map::*;
pub use self::resources::*;
pub use self::states::*;
pub use self::systems::*;
pub use self::utils::*;

// Bridge between bracket-lib and minigene
struct State {
    pub world: World,
    pub dispatcher: Box<dyn UnifiedDispatcher + 'static>,
    pub state_machine: StateMachine,
}
impl GameState for State {
    fn tick(&mut self, ctx: &mut BTerm) {
        if self.world.read_resource::<QuitGame>().0 {
            ctx.quitting = true;
        }
        if self.state_machine.is_running() && !ctx.quitting {
            mini_frame(
                &mut self.world,
                &mut self.dispatcher,
                ctx,
                &mut self.state_machine,
            );
        }
    }
}

fn main() -> BError {
    // Load spritesheet
    EMBED.lock().add_resource(SPRITESHEET_PATH.to_string(), include_bytes!("../assets/tilemap/colored_tilemap_packed.png"));
    let mut world = World::new();
    let builder = dispatcher!(
        world,
        (CombineCollisionSystem, "combine_collision", &[]),
        (InputDriver::<InputEvent>, "input_driver", &[]),
        (
            UpdateCollisionResourceSystem,
            "update_collision_res",
            &["combine_collision"],
        ),
        (CreepSpawnerSystem, "creep_spawner", &[]),
        (AiPathingSystem, "ai_pathing", &["update_collision_res"]),
        (AiMovementSystem, "ai_movement", &["ai_pathing"]),
        (ToggleGameSpeedSystem, "toggle_speed", &["input_driver"]),
        (WinConditionSystem, "win_cond", &[]),
        (SimpleMovementSystem, "simple_movement", &[]),
        (TowerAiSystem, "tower_ai", &[]),
        (ProximityAttackSystem, "proximity_attack", &[]),
        (TowerProjectileSystem, "tower_projectile", &[]),
        (UpdateEnemiesAroundSystem, "update_enemies_around", &[]),
        (SkillCooldownSystem::<Skills>, "cooldown_system", &[]),
        (TriggerPassiveSkillSystem::<Stats, Effectors, Skills, Items, (), ()>, "trigger_passives", &[]),
        (ExecSkillSystem::<Stats, Effectors, Skills, Items>, "exec_skills", &[]),
        (ApplyEffectorSystem::<Stats, Effectors>, "apply_effectors", &[]),
        (RemoveOutdatedEffectorSystem<Effectors>, "remove_effectors", &[]),
        (AoeDamageSystem, "aoe_damage", &[]),
        (GotoStraightSystem, "goto_straight", &[]),
        (HeroTeleportSystem, "hero_teleport", &[]),
        (QuitGameSystem, "quit_game", &[])
    );
    let mut spritesheet = SpriteSheet::new(SPRITESHEET_PATH);
    for j in 0..10 {
        for i in 0..10 {
            spritesheet = spritesheet.add_sprite(
                Rect::with_size(i*8, (9-j)*8, 8, 8)
            );
        }
    }
    let (mut world, mut dispatcher, mut context) =
        mini_init(SCREEN_WIDTH, SCREEN_HEIGHT, "Shotcaller", Some(spritesheet), builder, world);

    let mut state_machine = StateMachine::new(DefaultState);
    state_machine.start(&mut world, &mut dispatcher, &mut context);

    world.register::<MultiSprite>();
    world.register::<Sprite>();
    world.register::<Team>();
    world.register::<Barrack>();
    world.register::<Tower>();
    world.register::<Core>();
    world.register::<Leader>();
    world.register::<Name>();
    world.register::<SpriteIndex>();
    world.register::<Comp<StatSet<Stats>>>();
    world.register::<Comp<EffectorSet<Effectors>>>();
    world.register::<Comp<SkillSet<Skills>>>();
    world.register::<Comp<Inventory<Items, (), ()>>>();

    // WASM REGISTER
    world.register::<Point>();
    world.register::<SimpleMovement>();
    world.register::<AiPath>();
    world.register::<AiDestination>();
    world.register::<Creep>();
    world.register::<Player>();
    world.register::<CollisionMap>();
    world.register::<CreepSpawner>();
    world.register::<Collision>();
    world.register::<ProximityAttack>();
    world.register::<TowerProjectile>();
    world.register::<GotoStraight>();
    world.register::<GotoEntity>();
    world.insert(GameSpeed::default());
    world.insert(Winner::None);
    world.insert(QuitGame::default());

    let mut input_channel = EventChannel::<VirtualKeyCode>::new();
    let reader = input_channel.register_reader();
    world.insert(input_channel);
    world.insert(InputDriverRes(reader));

    let mut keymap = HashMap::new();
    keymap.insert(VirtualKeyCode::Escape, InputEvent::Quit);
    keymap.insert(VirtualKeyCode::J, InputEvent::MenuSouth);
    keymap.insert(VirtualKeyCode::K, InputEvent::MenuNorth);
    keymap.insert(VirtualKeyCode::H, InputEvent::MenuWest);
    keymap.insert(VirtualKeyCode::L, InputEvent::MenuEast);
    keymap.insert(VirtualKeyCode::Return, InputEvent::MenuSelect);
    keymap.insert(VirtualKeyCode::Q, InputEvent::MenuCancel);
    keymap.insert(VirtualKeyCode::S, InputEvent::SpeedToggle);
    keymap.insert(VirtualKeyCode::Key1, InputEvent::Teleport(1));
    keymap.insert(VirtualKeyCode::Key2, InputEvent::Teleport(2));
    keymap.insert(VirtualKeyCode::Key3, InputEvent::Teleport(3));
    keymap.insert(VirtualKeyCode::Key4, InputEvent::Teleport(4));
    keymap.insert(VirtualKeyCode::Key5, InputEvent::Teleport(5));
    world.insert(keymap);

    let mut input_channel = EventChannel::<InputEvent>::new();
    let reader = input_channel.register_reader();
    let reader2 = input_channel.register_reader();
    let reader3 = input_channel.register_reader();
    world.insert(input_channel);
    world.insert(ToggleGameSpeedRes(reader));
    world.insert(HeroTeleportRes{reader: reader2, selected_hero: None});
    world.insert(QuitGameRes(reader3));

    let mut skill_channel = EventChannel::<SkillTriggerEvent<Skills>>::new();
    let reader = skill_channel.register_reader();
    let reader2 = skill_channel.register_reader();
    world.insert(skill_channel);
    world.insert(ExecSkillRes(reader));
    world.insert(AoeDamageRes(reader2));

    world.insert(Camera::new(
        Point::new(0, 0),
        Point::new(PLAY_WIDTH, PLAY_HEIGHT),
    ));

    let stat_defs = StatDefinitions::from(vec![
        StatDefinition::new(
            Stats::Health,
            String::from("health"),
            String::from("HP"),
            100.0,
        ),
        StatDefinition::new(
            Stats::Defense,
            String::from("defense"),
            String::from("Defense"),
            0.0,
        ),
        StatDefinition::new(
            Stats::Attack,
            String::from("attack"),
            String::from("Attack"),
            10.0,
        ),
        StatDefinition::new(
            Stats::EnemiesAround,
            String::from("enemies_around"),
            String::from("Enemies Around"),
            0.0,
        ),
        StatDefinition::new(
            Stats::AttacksDealt,
            String::from("attacks_dealt"),
            String::from("Attacks Dealt"),
            0.0,
        ),
        StatDefinition::new(
            Stats::AttackSpeed,
            String::from("attack_speed"),
            String::from("Attack Speed"),
            10.0,
        ),
        StatDefinition::new(Stats::Mana, String::from("mana"), String::from("MP"), 100.0),
    ]);
    let default_stats = stat_defs.to_statset();

    let skill_definitions = SkillDefinitions::<Stats, Effectors, Skills, Items>::from(vec![
        SkillDefinition::new(
            Skills::AOE,
            String::from("AOE"),
            String::from("aoe"),
            String::from("Does 100 damage to all enemy entities around. Actives only if 3 or more enemy entities are present. Cooldown of 12s."),
            12.0,
            true,
            vec![
                // enemies around >= 3
                StatCondition::new(
                    Stats::EnemiesAround,
                    StatConditionType::MinValue(3.0),
                ),
            ],
            vec![],
            vec![],
        ),
        SkillDefinition::new(
            Skills::DoubleDamage,
            String::from("Double Damage"),
            String::from("double_damage"),
            String::from("Each 3 attacks, deal double damage."),
            0.0,
            true,
            vec![
                StatCondition::new(
                    Stats::AttacksDealt,
                    StatConditionType::Custom(|v| v as i32 % 3 == 0),
                ),
            ],
            vec![],
            vec![
                Effectors::DoubleDamage,
            ],
        ),
    ]);
    world.insert(skill_definitions);

    let effector_defs = EffectorDefinitions::from(vec![
        EffectorDefinition::new(
            Effectors::DoubleDamage,
            Some(0.0),
            vec![(Stats::Attack, EffectorType::MultiplicativeMultiplier(2.0))],
        ),
    ]);
    world.insert(effector_defs);

    let item_defs = ItemDefinitions::<_, _, ()>::from(vec![
        ItemDefinition::new(
            Items::TestItem,
            (),
            String::from("Test Item"),
            String::from("test_item"),
            String::from("A simple test item to check conditions."),
            None,
            None,
        ),
    ]);
    world.insert(item_defs);

    world.insert(stat_defs);
    world.insert(CollisionResource::new(
        CollisionMap::new(PLAY_WIDTH, PLAY_HEIGHT),
        Point::new(0, 0),
    ));

    // Create cores
    world
        .create_entity()
        .with(Point::new(PLAY_WIDTH as i32 / 2, 1))
        .with(Sprite {
            glyph: to_cp437('C'),
            fg: RGBA::named(BLUE),
            bg: RGBA::named(RED),
        })
        .with(SpriteIndex(66))
        .with(Team::Other)
        .with(Core)
        .with(Comp(default_stats.clone()))
        .build();

    world
        .create_entity()
        .with(Point::new(PLAY_WIDTH as i32 / 2, PLAY_HEIGHT as i32 - 2))
        .with(Sprite {
            glyph: to_cp437('C'),
            fg: RGBA::named(BLUE),
            bg: RGBA::named(GREEN),
        })
        .with(SpriteIndex(66))
        .with(Team::Me)
        .with(Core)
        .with(Comp(default_stats.clone()))
        .build();

    // Create barracks
    for i in -1..=1 {
        let x = PLAY_WIDTH as i32 / 2 + PLAY_WIDTH as i32 / 7 * i as i32;
        let y = PLAY_HEIGHT as i32 / 8;
        world
            .create_entity()
            .with(Point::new(x, y))
            .with(Sprite {
                glyph: to_cp437('B'),
                fg: RGBA::named(YELLOW),
                bg: RGBA::named(RED),
            })
            .with(SpriteIndex(69))
            .with(Team::Other)
            .with(Barrack)
            .with(Comp(default_stats.clone()))
            .build();
        // Creep spawners
        world
            .create_entity()
            .with(Point::new(x, y + 1))
            .with(CreepSpawner(0, CREEP_SPAWN_TICKS))
            //.with(CreepSpawner(0, 2))
            .with(Team::Other)
            .build();
    }

    for i in -1..=1 {
        let x = PLAY_WIDTH as i32 / 2 + PLAY_WIDTH as i32 / 7 * i;
        let y = PLAY_HEIGHT as i32 - 1 - PLAY_HEIGHT as i32 / 8;
        world
            .create_entity()
            .with(Point::new(x, y))
            .with(Sprite {
                glyph: to_cp437('B'),
                fg: RGBA::named(YELLOW),
                bg: RGBA::named(GREEN),
            })
            .with(SpriteIndex(69))
            .with(Team::Me)
            .with(Barrack)
            .with(Comp(default_stats.clone()))
            .build();
        // Creep spawners
        world
            .create_entity()
            .with(Point::new(x, y - 1))
            .with(CreepSpawner(0, CREEP_SPAWN_TICKS))
            .with(Team::Me)
            .build();
    }

    // Create towers
    for i in -1..=1 {
        for j in 1..=2 {
            world
                .create_entity()
                .with(Point::new(
                    PLAY_WIDTH as i32 / 2 + PLAY_WIDTH as i32 / 4 * i,
                    PLAY_HEIGHT as i32 * j / 6,
                ))
                .with(Sprite {
                    glyph: to_cp437('T'),
                    fg: RGBA::named(GREEN),
                    bg: RGBA::named(RED),
                })
                .with(SpriteIndex(80))
                .with(Team::Other)
                .with(Comp(default_stats.clone()))
                .build();
        }
    }

    for i in -1..=1 {
        for j in 1..=2 {
            world
                .create_entity()
                .with(Point::new(
                    PLAY_WIDTH as i32 / 2 + PLAY_WIDTH as i32 / 4 * i,
                    PLAY_HEIGHT as i32 - 1 - PLAY_HEIGHT as i32 * j / 6,
                ))
                .with(Sprite {
                    glyph: to_cp437('T'),
                    fg: RGBA::named(GREEN),
                    bg: RGBA::named(GREEN),
                })
                .with(SpriteIndex(80))
                .with(Team::Me)
                .with(Comp(default_stats.clone()))
                .build();
        }
    }

    // hero1 skill set
    let mut skillset = SkillSet::new(HashMap::new());
    skillset.skills.insert(Skills::DoubleDamage, SkillInstance::new(Skills::DoubleDamage, 0.0));
    skillset.skills.insert(Skills::AOE, SkillInstance::new(Skills::AOE, 0.0));

    let default_inventory = Inventory::<Items, (), ()>::new_fixed(4);

    // TODO re-enable de the hero
    // currently disabled to make the game balanced
    // Create generic hero 1
    /*let hero1 = world
        .create_entity()
        .with(Point::new(PLAY_WIDTH as i32 / 2, PLAY_HEIGHT as i32 - 11))
        .with(Sprite {
            glyph: to_cp437('L'),
            //fg: RGBA::named(YELLOW),
            fg: RGBA::named(RED),
            bg: RGBA::named(GREEN),
        })
        .with(SpriteIndex(6))
        .with(Team::Me)
        .with(SimpleMovement)
        .with(Comp(default_inventory.clone()))
        .with(Comp(skillset))
        .with(AiPath::new(NavigationPath::new()))
        .with(Leader(1))
        .with(ProximityAttack::new(LEADER_ATTACK_RADIUS))
        .with(Name("Generic Leader 1".to_string()))
        .with(Comp(default_stats.clone()))
        .with(Comp(EffectorSet::<Effectors>::default()))
        .build();*/

    // Make hero HP really high. Used for testing win conditions.
    //world.write_storage::<Comp<StatSet<Stats>>>().get_mut(hero1).unwrap().0.stats.get_mut(&Stats::Health).unwrap().value = 10000000.0;

    create_map_bg(&mut world);

    let gs = State {
        world,
        dispatcher,
        state_machine,
    };

    main_loop(context, gs)
}
