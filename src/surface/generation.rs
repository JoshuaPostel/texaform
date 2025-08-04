use std::collections::{BTreeMap, HashSet};

use noise::utils::{NoiseMapBuilder, PlaneMapBuilder};
use noise::{self};
use rand::Rng;
use rand::SeedableRng;
use rand_chacha::ChaCha8Rng;

use tokio::sync::mpsc::UnboundedSender;

use crate::agents::Agent;
use crate::entities::Properties;
use crate::event::Event;
use crate::tech_tree::TechTree;

use crate::utils::{idx_to_pos, pos_to_idx};

use crate::agents::dog::Dog;
use crate::agents::fabricator::Fabricator;
use crate::agents::hud::Hud;
use crate::agents::laser_cutter::LaserCutter;
use crate::agents::smelter::Smelter;
use crate::entities::shape::Shape;

use crate::surface::grid::{Gent, Grid};
use crate::surface::{AddEntityError, Focus, GRID_SIZE, GameState, GameStats, Power, Surface};

use ratatui::layout::Position;

fn insert_shape(
    shape: Shape,
    prop: Properties,
    grid: &mut Vec<Gent>,
    idx: usize,
    unbuildable_idx: &mut HashSet<usize>,
) {
    let grid_pos = idx_to_pos(idx, GRID_SIZE);
    let mut idxs = HashSet::new();
    for offset in &shape.positions {
        let adj_x = grid_pos.x + offset.x;
        // avoids inserting shapes that wrapp around east/west boarder
        if adj_x == 0 || adj_x as usize == GRID_SIZE {
            return;
        }
        let adj_pos = Position::new(adj_x, grid_pos.y + offset.y);
        let grid_idx = pos_to_idx(&adj_pos, GRID_SIZE);
        idxs.insert(grid_idx);
    }
    if unbuildable_idx.intersection(&idxs).count() > 0
        || idxs.iter().any(|i| *i > GRID_SIZE * GRID_SIZE)
    {
        return;
    }
    for grid_idx in idxs {
        grid[grid_idx] = Gent::Intmd(prop);
    }
    let footprint = shape.grid_footprint(&grid_pos, GRID_SIZE);
    unbuildable_idx.extend(footprint);
}

pub async fn manual(event_sender: UnboundedSender<Event>, seed: u64) -> Surface {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let mut grid: Vec<Gent> = vec![];
    for _ in 0..(GRID_SIZE * GRID_SIZE) {
        grid.push(Gent::Empty)
    }

    let mut unbuildable = HashSet::<usize>::new();
    let starting_area = Shape::circle(10);
    let center_adj = ((GRID_SIZE / 2) - 5) as u16;
    for offset in starting_area.positions {
        let pos = Position::new(offset.x + center_adj, offset.y + center_adj);
        unbuildable.insert(pos_to_idx(&pos, GRID_SIZE));
    }

    for idx in 0..(GRID_SIZE * GRID_SIZE) {
        if rng.random::<f64>() > 0.997 {
            let radius = rng.random_range(4..7);
            let iters = rng.random_range(2..5);
            let copper_vein = Shape::jittered_circle(&mut rng, radius, iters);
            insert_shape(
                copper_vein,
                Properties::Copper,
                &mut grid,
                idx,
                &mut unbuildable,
            );
        }
    }

    for idx in 0..(GRID_SIZE * GRID_SIZE) {
        if rng.random::<f64>() > 0.998 {
            let length = rng.random_range(4..10);
            let bend_chance = rng.random_range(0.4..0.6);
            let horizontal = rng.random::<bool>();
            let wf = Shape::waffle_fry(&mut rng, length, bend_chance, horizontal).translate(0, 1);
            insert_shape(wf, Properties::Silicate, &mut grid, idx, &mut unbuildable);
        }
    }

    for idx in 0..(GRID_SIZE * GRID_SIZE) {
        if rng.random::<f64>() > 0.995 {
            insert_shape(
                Shape::diamond(),
                Properties::Sulfer,
                &mut grid,
                idx,
                &mut unbuildable,
            );
        }
    }

    for idx in 0..(GRID_SIZE * GRID_SIZE) {
        if rng.random::<f64>() > 0.90 && !unbuildable.contains(&idx) {
            grid[idx] = Gent::Intmd(Properties::Iron);
        }
    }

    let grid = Grid::new(grid);

    let x = (GRID_SIZE / 2) - 30;
    let y = (GRID_SIZE / 2) - 10;
    Surface::new(grid, x, y, event_sender).await
}

pub fn perlin(event_sender: UnboundedSender<Event>) -> Surface {
    let noise = noise::Perlin::new(42);
    //let noise = noise::BasicMulti::<BasicMulti<Perlin>>::new(42);
    //let noise = noise::Billow::<noise::Perlin>::new(42);
    //let noise = noise::Fbm::<noise::Worley>::default();
    //let noise = noise::Cylinders::new().set_frequency(10.0);
    //
    // TODO try using 10x or 100x larger plane map builder and then sample from it every 10/100
    // places (should hopfully "scale down"/"make smaller" the pattern)
    tracing::info!("start plane");
    let plane = PlaneMapBuilder::new(noise)
        .set_size(GRID_SIZE, GRID_SIZE)
        .build();
    tracing::info!("end plane");

    for val in plane.iter().take(5) {
        tracing::info!("HERE: {val}");
    }

    let mut grid: Vec<Gent> = vec![];
    for val in plane.iter() {
        if *val > 0.25 {
            grid.push(Gent::Intmd(Properties::Iron));
        } else {
            grid.push(Gent::Empty)
        }
    }
    grid[GRID_SIZE] = Gent::Empty;
    grid[GRID_SIZE + 1] = Gent::Empty;

    for x in 0..15 {
        for y in 1..15 {
            grid[GRID_SIZE * y + x] = Gent::Empty;
        }
    }

    let grid = Grid::new(grid);
    let game_state = GameState {
        unlocked_entities: HashSet::new(),
        tech_tree: TechTree::new(),
        tutorial_state: Default::default(),
    };

    Surface {
        grid,
        x: 0,
        y: 0,
        agents: BTreeMap::new(),
        event_sender,
        game_state,
        game_stats: GameStats::default(),
        victory_stats: None,
        power: Power::default(),
        effects: vec![],
        focus: None,
        hud: Hud::default(),
    }
}

pub fn empty(event_sender: UnboundedSender<Event>) -> Surface {
    let game_state = GameState {
        unlocked_entities: HashSet::new(),
        tech_tree: TechTree::new(),
        tutorial_state: Default::default(),
    };
    let grid: Vec<Gent> = vec![];
    let grid = Grid::new(grid);

    Surface {
        grid,
        x: 0,
        y: 0,
        agents: BTreeMap::new(),
        event_sender,
        game_state,
        game_stats: GameStats::default(),
        victory_stats: None,
        power: Power::default(),
        effects: vec![],
        focus: None,
        hud: Hud::default(),
    }
}

pub async fn init_starting_entities(surface: &mut Surface) -> Result<(), AddEntityError> {
    let mut fab = Fabricator::new();

    // push items needed to make smelter
    let smelter_cost = Properties::Smelter.cost().expect("has cost");
    for (prop, count) in smelter_cost.into_iter() {
        for _ in 0..count {
            fab.place(prop)
        }
    }

    let lc_cost = Properties::LaserCutter.cost().expect("has cost");
    for (prop, count) in lc_cost.into_iter() {
        for _ in 0..count {
            fab.place(prop)
        }
    }

    let center = GRID_SIZE as u16 / 2;
    surface
        .add_agent(&Position::new(center, center), Box::new(fab))
        .await?;

    let dog = Dog::new();
    surface
        .add_agent(&Position::new(center - 2, center - 2), Box::new(dog))
        .await?;
    surface.focus = Some(Focus::Agent(3335));
    let dog = Dog::new();
    surface
        .add_agent(&Position::new(center - 2, center), Box::new(dog))
        .await?;

    surface.add_entity(&Position::new(center + 1, center - 2), Properties::Iron)?;
    surface.add_entity(&Position::new(center + 2, center - 2), Properties::Iron)?;
    surface.add_entity(&Position::new(center + 5, center), Properties::SolarPannel)?;
    surface.add_entity(
        &Position::new(center + 5, center + 2),
        Properties::Accumulator,
    )?;

    Ok(())
}

pub async fn init_starting_agent(surface: &mut Surface) -> Result<(), AddEntityError> {
    let mut fab = Fabricator::new();

    // push items needed to make dog
    let dog_cost = Properties::Dog.cost().expect("dog has cost");
    for (prop, count) in dog_cost.into_iter() {
        for _ in 0..count {
            fab.place(prop)
        }
    }

    // push items needed to make smelter
    let smelter_cost = Properties::Smelter.cost().expect("smelter has cost");
    //let mut x = 0;
    for (prop, count) in smelter_cost.into_iter() {
        for _ in 0..count {
            fab.buffer_in.content.push(prop)
            //let pos = Position::new(grid_center + (x), grid_center - 2);
            //x += 1;
            //surface.add_entity(&pos, Gent::Intmd(prop))?;
        }
    }

    let grid_center = GRID_SIZE as u16 / 2;
    surface
        .add_agent(&Position::new(grid_center, grid_center), Box::new(fab))
        .await?;

    let solar_pannel = Properties::SolarPannel;
    let sp1 = Position {
        x: grid_center,
        y: grid_center + 4,
    };
    let sp2 = Position {
        x: grid_center,
        y: grid_center + 6,
    };
    surface.add_entity(&sp1, solar_pannel)?;
    surface.add_entity(&sp2, solar_pannel)?;

    let accumulator = Properties::Accumulator;
    let acc1 = Position {
        x: grid_center,
        y: grid_center + 8,
    };
    let acc2 = Position {
        x: grid_center + 3,
        y: grid_center + 8,
    };
    surface.add_entity(&acc1, accumulator)?;
    surface.add_entity(&acc2, accumulator)?;

    Ok(())
}

pub async fn init_some_agents(surface: &mut Surface) -> Result<(), AddEntityError> {
    let dog = Dog::new();
    surface
        .add_agent(&Position { x: 2, y: 7 }, Box::new(dog))
        .await?;

    let mut smelter = Smelter::new();
    smelter.hearth = Some(Properties::Iron);
    surface
        .add_agent(&Position { x: 1, y: 8 }, Box::new(smelter))
        .await?;

    let laser_cutter = LaserCutter::new();
    surface
        .add_agent(&Position { x: 9, y: 2 }, Box::new(laser_cutter))
        .await?;

    let solar_pannel = Properties::SolarPannel;
    let sp1 = Position { x: 2, y: 12 };
    let sp2 = Position { x: 6, y: 12 };
    surface.add_entity(&sp1, solar_pannel)?;
    surface.add_entity(&sp2, solar_pannel)?;
    let accumulator = Properties::Accumulator;
    let acc1 = Position { x: 2, y: 13 };
    let acc2 = Position { x: 4, y: 13 };
    let acc3 = Position { x: 7, y: 13 };
    surface.add_entity(&acc1, accumulator)?;
    surface.add_entity(&acc2, accumulator)?;
    surface.add_entity(&acc3, accumulator)?;

    Ok(())
}
