import cr_trichome as crt

if __name__ == "__main__":
    settings = crt.SimulationSettings()
    print(settings)

    crt.run_sim(settings)

    crt.plot_cells_at_all_iterations(0.0, 4.0, overwrite=True, transparent=True)
