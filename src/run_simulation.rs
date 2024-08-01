use std::num::NonZeroUsize;

use cellular_raza::core::backend::chili;
use cellular_raza::core::time::FixedStepsize;
use cellular_raza::prelude::*;

use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde::{Deserialize, Serialize};

use crate::cell_properties::*;
use crate::custom_domain::*;

use pyo3::prelude::*;

#[pyclass]
#[derive(Clone, Debug)]
pub struct SimulationSettings {
    pub cell_mechanics_area: f64,
    pub cell_mechanics_spring_tension: f64,
    pub cell_mechanics_central_pressure: f64,
    pub cell_mechanics_interaction_range: f64,
    pub cell_mechanics_potential_strength: f64,
    pub cell_mechanics_damping_constant: f64,
    pub cell_mechanics_diffusion_constant: f64,
    pub domain_size: f64,
    pub n_times: u64,
    pub dt: f64,
    pub t_start: f64,
    pub save_interval: u64,
    pub n_threads: NonZeroUsize,
    pub seed: u64,
}

impl Default for SimulationSettings {
    fn default() -> Self {
        Self {
            cell_mechanics_area: 500.0,
            cell_mechanics_spring_tension: 2.0,
            cell_mechanics_central_pressure: 0.5,
            cell_mechanics_interaction_range: 5.0,
            cell_mechanics_potential_strength: 6.0,
            cell_mechanics_damping_constant: 0.2,
            cell_mechanics_diffusion_constant: 0.0,

            // Parameters for domain
            domain_size: 800.0,

            // Time parameters
            n_times: 20_001,
            dt: 0.005,
            t_start: 0.0,
            save_interval: 50,

            // Meta Parameters to control solving
            n_threads: 1.try_into().unwrap(),
            seed: 2,
        }
    }
}

#[pymethods]
impl SimulationSettings {
    pub fn __repr__(&self) -> String {
        format!("{:#?}", self)
    }

    #[new]
    #[pyo3(signature = (
        cell_mechanics_area=500.0,
        cell_mechanics_spring_tension=2.0,
        cell_mechanics_central_pressure=0.5,
        cell_mechanics_interaction_range=5.0,
        cell_mechanics_potential_strength=6.0,
        cell_mechanics_damping_constant=0.2,
        cell_mechanics_diffusion_constant=0.0,
        domain_size=800.0,
        n_times=20001,
        dt=0.005,
        t_start=0.0,
        save_interval=50,
        n_threads=1,
        seed=2,
    ))]
    pub fn new(
        cell_mechanics_area: f64,
        cell_mechanics_spring_tension: f64,
        cell_mechanics_central_pressure: f64,
        cell_mechanics_interaction_range: f64,
        cell_mechanics_potential_strength: f64,
        cell_mechanics_damping_constant: f64,
        cell_mechanics_diffusion_constant: f64,
        domain_size: f64,
        n_times: u64,
        dt: f64,
        t_start: f64,
        save_interval: u64,
        n_threads: usize,
        seed: u64,
    ) -> Self {
        Self {
            cell_mechanics_area,
            cell_mechanics_spring_tension,
            cell_mechanics_central_pressure,
            cell_mechanics_interaction_range,
            cell_mechanics_potential_strength,
            cell_mechanics_damping_constant,
            cell_mechanics_diffusion_constant,
            domain_size,
            n_times,
            dt,
            t_start,
            save_interval,
            n_threads: n_threads.try_into().unwrap(),
            seed,
        }
    }

    #[staticmethod]
    pub fn default() -> Self {
        <Self as Default>::default()
    }
}

#[pyfunction]
pub fn run_sim(settings: SimulationSettings) -> Result<(), chili::SimulationError> {
    // Fix random seed
    let mut rng = ChaCha8Rng::seed_from_u64(settings.seed);

    // Define the simulation domain
    let domain = MyDomain {
        cuboid: CartesianCuboid::from_boundaries_and_interaction_range(
            [0.0; 2],
            [settings.domain_size; 2],
            2.0 * VertexMechanics2D::<6>::inner_radius_from_cell_area(settings.cell_mechanics_area),
        )?,
    };

    // Define cell agents
    let models = VertexMechanics2D::fill_rectangle_flat_top(
        settings.cell_mechanics_area,
        settings.cell_mechanics_spring_tension,
        settings.cell_mechanics_central_pressure,
        settings.cell_mechanics_damping_constant,
        settings.cell_mechanics_diffusion_constant,
        [
            [0.1 * settings.domain_size; 2].into(),
            [0.9 * settings.domain_size; 2].into(),
        ],
    );
    println!("Generated {} cells", models.len());

    let k1 = 0.6662;
    let k2 = 0.1767;
    let k3 = 3.1804;
    let k4 = 5.3583;
    let k5 = 1.0;
    // let contact_range = (CELL_MECHANICS_AREA / std::f64::consts::PI).sqrt() * 1.5;
    let contact_range = 0.9 * settings.domain_size / (models.len() as f64).sqrt() * 1.5;
    let f = -((k1 * k4 - 1f64).powf(2.0) - 4.0 * k2 * k4 * k5).sqrt();
    let v0 = nalgebra::vector![
        (k1 * k4 - 1.0 + f) / (2.0 * k2 * k4),
        (k1 * (k1 * k4 - 1.0 - f) - 2.0 * k2 * k5) / (2.0 * k5),
        (k1 * k4 + 1.0 - f) / (2.0 * k4),
    ];
    let mechanics_area_threshold = settings.cell_mechanics_area * 2.0;
    let growth_rate = 0.01;
    let cells = models
        .into_iter()
        .map(|model| MyCell {
            mechanics: model,
            interaction: VertexDerivedInteraction::from_two_forces(
                OutsideInteraction {
                    potential_strength: settings.cell_mechanics_potential_strength,
                    interaction_range: settings.cell_mechanics_interaction_range,
                },
                InsideInteraction {
                    potential_strength: 1.5 * settings.cell_mechanics_potential_strength,
                    average_radius: settings.cell_mechanics_area.sqrt(),
                },
            ),
            intracellular: nalgebra::vector![
                rng.gen_range(0.9 * v0[0]..1.1 * v0[0]),
                rng.gen_range(0.9 * v0[1]..1.1 * v0[1]),
                rng.gen_range(0.9 * v0[2]..1.1 * v0[2]),
            ],
            k1,
            k2,
            k3,
            k4,
            k5,
            contact_range,
            mechanics_area_threshold,
            growth_rate,
        })
        .collect::<Vec<_>>();

    // Define settings for storage and time solving
    let settings = chili::Settings {
        time: FixedStepsize::from_partial_save_steps(
            0.0,
            settings.dt,
            settings.n_times,
            settings.save_interval,
        )?,
        n_threads: settings.n_threads,
        show_progressbar: true,
        storage: StorageBuilder::new()
            .location("out/cr_trichome")
            .priority([StorageOption::SerdeJson]),
    };

    // Run the simulation
    let _storager = chili::run_simulation!(
        agents: cells,
        domain: domain,
        settings: settings,
        aspects: [Reactions, ReactionsContact],
    )?;
    Ok(())
}
