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

/// This class contains all settings needed to run a full simulation with the `run_sim` function.
///
/// Attributes
/// ----------
/// cell_mechanics_area(float):
///     Defines the total size of each cell. Currently all cells are assigned identical sizes.
/// cell_mechanics_spring_tension(float):
///     Spring constant of the edges of the cell.
/// cell_mechanics_central_pressure(float):
///     Internal pressure which pushes vertices outwards from the middle.
/// cell_mechanics_interaction_range(float):
///     Maximal interaction range until which other cells will be attracted via an outside
///     potential.
///     This value is also used to calculate the discretization of the total simulation domain.
/// cell_mechanics_potential_strength(float):
///     Interaction strength for repelling and attracting strength between the cells.
/// cell_mechanics_damping_constant(float):
///     Damping constant $\lambda$ for the physical mechanics of the cell.
/// cell_mechanics_diffusion_constant(float):
///     Amplitude of the Wiener process in the phyical mechanics of the cell.
/// domain_size(float):
///     Total size of the simulation quadratically-sized domain.
/// n_times(int):
///     Number of integration steps to take.
/// dt(float):
///     Temporal discretization used for solving all equations.
/// t_start(float):
///     Initial time point at which the simulation is started.
/// save_interval(int):
///     Every nth step will be saved to the output folder.
/// n_threads(int):
///     Number of threads to use for parallelization.
/// seed(int):
///     Initial seed of random number generator for the simulation.
#[pyclass(get_all, set_all)]
#[derive(Clone, Debug, Serialize, Deserialize, approx::AbsDiffEq, PartialEq)]
pub struct SimulationSettings {
    pub cell_mechanics_area: f64,
    pub cell_mechanics_spring_tension: f64,
    pub cell_mechanics_central_pressure: f64,
    pub cell_mechanics_interaction_range: f64,
    pub cell_mechanics_potential_strength: f64,
    pub cell_mechanics_damping_constant: f64,
    pub cell_mechanics_diffusion_constant: f64,
    pub cell_growth_rate: f64,
    pub domain_size: f64,
    #[approx(equal)]
    pub n_voxels: usize,
    #[approx(equal)]
    pub n_times: u64,
    pub dt: f64,
    pub t_start: f64,
    #[approx(equal)]
    pub save_interval: u64,
    #[approx(equal)]
    pub n_threads: NonZeroUsize,
    #[approx(equal)]
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
            cell_growth_rate: 0.005,

            // Parameters for domain
            domain_size: 800.0,
            n_voxels: 10,

            // Time parameters
            n_times: 40_001,
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
        cell_growth_rate=0.005,
        domain_size=800.0,
        n_voxels=10,
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
        cell_growth_rate: f64,
        domain_size: f64,
        n_voxels: usize,
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
            cell_growth_rate,
            domain_size,
            n_voxels,
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

fn compare_sims(settings: &SimulationSettings, f: &std::path::Path) -> Option<std::path::PathBuf> {
    let fpath = f.to_path_buf();
    let mut spath = fpath.clone();
    spath.push("settings.ron");
    let settings_str = std::fs::read_to_string(&spath).ok()?;
    let settings2: SimulationSettings = ron::from_str(&settings_str).ok()?;
    if approx::abs_diff_eq!(settings, &settings2) {
        return Some(fpath);
    }
    None
}

fn find_simulation(settings: &SimulationSettings) -> Option<std::path::PathBuf> {
    for f in glob::glob("out/cr_trichome/*").ok()?.filter_map(|x| x.ok()) {
        if let Some(opath) = compare_sims(settings, &f) {
            return Some(opath);
        }
    }
    None
}

/// Parameters
/// ----------
/// settings : SimulationSettings
///     All settings which need to be specified to run a full simulation.
///
/// Raises:
///     ValueError : When the simulation aborts due to an unexpected error.
#[pyfunction]
pub fn run_sim(settings: SimulationSettings) -> Result<std::path::PathBuf, SimulationError> {
    if let Some(opath) = find_simulation(&settings) {
        println!("Loading Simulation from {}", opath.display());
        return Ok(opath);
    }

    // Fix random seed
    let mut rng = ChaCha8Rng::seed_from_u64(settings.seed);

    // Define the simulation domain
    let domain = MyDomain {
        cuboid: CartesianCuboid::from_boundaries_and_n_voxels(
            [0.0; 2],
            [settings.domain_size; 2],
            [settings.n_voxels; 2],
            // 2.0 * VertexMechanics2D::<6>::inner_radius_from_cell_area(settings.cell_mechanics_area),
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
    let f = ((k1 * k4 - 1f64).powf(2.0) - 4.0 * k2 * k4 * k5).sqrt();
    let v0 = nalgebra::vector![
        (k1 * k4 - 1.0 + f) / (2.0 * k2 * k4),
        (k1 * (k1 * k4 - 1.0 - f) - 2.0 * k2 * k5) / (2.0 * k5),
        (k1 * k4 + 1.0 - f) / (2.0 * k4),
    ];
    let mechanics_area_threshold = settings.cell_mechanics_area * 2.0;
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
                rng.random_range(0.9 * v0[0]..1.1 * v0[0]),
                rng.random_range(0.9 * v0[1]..1.1 * v0[1]),
                rng.random_range(0.9 * v0[2]..1.1 * v0[2]),
            ],
            k1,
            k2,
            k3,
            k4,
            k5,
            contact_range,
            mechanics_area_threshold,
            growth_rate: settings.cell_growth_rate,
        })
        .collect::<Vec<_>>();

    // Define settings for storage and time solving
    let chili_settings = chili::Settings {
        time: FixedStepsize::from_partial_save_steps(
            0.0,
            settings.dt,
            settings.n_times,
            settings.save_interval,
        )?,
        n_threads: settings.n_threads,
        progressbar: Some("".to_string()),
        storage: StorageBuilder::new()
            .location("out/cr_trichome")
            .priority([StorageOption::SerdeJson]),
    };

    // Run the simulation
    let storager = chili::run_simulation!(
        agents: cells,
        domain: domain,
        settings: chili_settings,
        aspects: [Reactions, ReactionsContact],
    )?;

    // Store settings in output folder
    let mut opath = storager.cells.extract_builder().get_full_path();
    opath.pop();
    let mut settings_path = opath.clone();
    settings_path.push("settings.ron");
    let ron_pretty_config = ron::ser::PrettyConfig::default();
    let settings_str = ron::ser::to_string_pretty(&settings, ron_pretty_config)
        .map_err(|e| chili::SimulationError::StorageError(e.into()))?;
    std::fs::write(&settings_path, settings_str)?;

    Ok(opath)
}
