import cr_trichome as crt
import matplotlib.pyplot as plt
from matplotlib.gridspec import GridSpec
import numpy as np
from tqdm import tqdm
import string


COLOR1 = "#6bd2db"
COLOR2 = "#0ea7b5"
COLOR3 = "#0c457d"
COLOR4 = "#ffbe4f"
COLOR5 = "#e8702a"
COLOR6 = "#a02b08"


def set_mpl_rc_params():
    plt.rcParams.update(
        {
            "font.family": "Courier New",  # monospace font
            "font.size": 25,
            "axes.titlesize": 25,
            "axes.labelsize": 25,
            "xtick.labelsize": 25,
            "ytick.labelsize": 25,
            "legend.fontsize": 25,
            "figure.titlesize": 25,
        }
    )


def configure_ax(ax, minor=True):
    ax.grid(True, which="major", linestyle="-", linewidth=0.75, alpha=0.25)
    ax.minorticks_on()
    if minor:
        ax.grid(True, which="minor", linestyle="-", linewidth=0.25, alpha=0.15)
    else:
        ax.grid(False, which="minor")
    ax.set_axisbelow(True)


def derivative(y, dt):
    return (y[:-4] - 8 * y[1:-3] + 8 * y[3:-1] - y[4:]) / (12 * dt)


if __name__ == "__main__":
    settings = crt.SimulationSettings.default()

    # crt.run_sim(settings)
    # crt.plot_cells_at_all_iterations(0.0, 4.0, overwrite=True, transparent=True)

    iterations = crt.get_all_iterations()[::20]  # TODO remove this thinning

    set_mpl_rc_params()
    fig = plt.figure(figsize=(24, 16))
    gs = GridSpec(2, 1)
    fig1 = fig.add_subfigure(gs[0])
    fig2 = fig.add_subfigure(gs[1])

    t = iterations * settings.dt

    concs = []
    for it in tqdm(iterations):
        cells = crt.load_cells(it)
        y = np.array([x for x in cells["cell.intracellular"]], dtype=float)
        concs.append(y)

    concs = np.array(concs)
    dconcs = derivative(concs, settings.dt)
    dconcs_mean = np.mean(dconcs, axis=1)
    dconcs_std = np.std(dconcs, axis=1)

    axs1 = fig1.subplots(1, 3)
    axs2 = fig2.subplots(1, 3)

    for ax in axs1:
        configure_ax(ax)

    n_peaks = np.sum(concs[:, :, 2] > 2.0, axis=1)
    axs1[0].plot(t, n_peaks, color=COLOR3, label="Number of Peaks")
    axs1[0].legend(frameon=False, loc="upper center", bbox_to_anchor=(0.5, 1.10))

    # axs1[2].plot(t[2:-2], dconcs_mean[:, 0], color=COLOR1)
    # axs1[2].plot(t[2:-2], dconcs_mean[:, 1], color=COLOR3)
    axs1[2].plot(t[2:-2], dconcs_mean[:, 2], color=COLOR3, label="Avg. Derivative ??")
    axs1[2].legend(frameon=False, loc="upper center", bbox_to_anchor=(0.5, 1.10))

    n_max = np.argmax(n_peaks)

    iters = [iterations[0], iterations[n_max], iterations[-1]]
    for it, ax, label in zip(iters, axs2, string.ascii_uppercase):
        cells = crt.load_cells(it)
        ax.set_xlim(50, 750)
        ax.set_ylim(50, 750)
        ax.set_axis_off()
        crt.plot_cells(ax, cells, 0.0, 4.0)
        ax.text(
            0.1,
            0.9,
            label,
            fontsize=40,
            fontweight="semibold",
            fontfamily="serif",
            va="top",
            horizontalalignment="left",
            transform=ax.transAxes,
            color="white",
        )

    fig1.subplots_adjust(
        left=0.03, right=0.97, bottom=0.07, top=0.93, wspace=0.20, hspace=0
    )
    fig2.subplots_adjust(left=0, right=1, bottom=0, top=1, wspace=0, hspace=0)
    fig.savefig("temp.pdf")
