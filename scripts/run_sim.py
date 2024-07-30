import cr_trichome as crt

if __name__ == "__main__":
    settings = crt.SimulationSettings()
    print(settings)

    crt.run_sim(settings)

