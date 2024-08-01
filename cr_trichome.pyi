class SimulationSettings:
    cell_mechanics_area: float
    cell_mechanics_spring_tension: float
    cell_mechanics_central_pressure: float
    cell_mechanics_interaction_range: float
    cell_mechanics_potential_strength: float
    cell_mechanics_damping_constant: float
    cell_mechanics_diffusion_constant: float
    domain_size: float
    n_times: int
    dt: float
    t_start: float
    save_interval: int
    n_threads: int
    seed: int

    @staticmethod
    def default() -> SimulationSettings:
        pass

def run_sim(settings: SimulationSettings):
    pass

