import cr_trichome as crt
import matplotlib.pyplot as plt
from matplotlib.gridspec import GridSpec
import numpy as np
from tqdm import tqdm
import string
import os
import argparse


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


def generate_movie(opath):
    cmd = f"ffmpeg -y  -pattern_type glob -i '{opath / 'images/*.png'}' -c:v libx264 movie.mp4"
    os.system(cmd)


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--movie", action="store_true", default=False)
    parser.add_argument("--images", action="store_true", default=False)
    pyargs = parser.parse_args()

    settings = crt.SimulationSettings.default()

    settings.n_threads = 2
    settings.n_times = 20_000
    settings.save_interval = 200

    settings.cell_growth_rate = 0.0

    opath = crt.run_sim(settings)

    iterations = crt.get_all_iterations(opath)

    set_mpl_rc_params()
    fig, axs = plt.subplots(1, 3, figsize=(24, 8))

    t = iterations * settings.dt

    concs = []
    for it in tqdm(iterations):
        cells = crt.load_cells(it, opath)
        y = np.array([x for x in cells["cell.intracellular"]], dtype=float)
        concs.append(y)

    concs = np.array(concs)
    conc_max = 6.0

    # Create Movie and Images
    if pyargs.movie or pyargs.images:
        crt.plot_cells_at_all_iterations(
            0.0, conc_max, settings.domain_size, opath, overwrite=True, transparent=True
        )
    if pyargs.movie:
        generate_movie(opath)

    for i, (ax, label) in enumerate(zip(axs, string.ascii_uppercase)):
        configure_ax(ax)
        ax.text(
            0.03,
            1 - 0.03,
            label,
            fontsize=40,
            fontweight="semibold",
            fontfamily="serif",
            va="top",
            horizontalalignment="left",
            transform=ax.transAxes,
            color="black" if i == 0 else "#fafcc0",
        )

    n_peaks = np.sum(concs[:, :, 2] > conc_max / 2, axis=1)
    np_ini = np.argmax(np.where(n_peaks < np.max(n_peaks) / 2))
    np_mid = int(len(iterations) / 2)
    np_fin = len(iterations) - 1

    for thresh, color in zip([0.45, 0.5, 0.55], [COLOR1, COLOR3, COLOR5]):
        n_peaks_thresh = np.sum(concs[:, :, 2] > conc_max * thresh, axis=1)
        axs[0].plot(
            t,
            n_peaks_thresh,
            color=color,
            label=f"Threshold={thresh:.2f}",
        )
    axs[0].set_title("Number of [AC] Peaks")
    axs[0].legend(frameon=False)

    axs[0].scatter(
        [t[np_ini], t[-1]],
        [n_peaks[np_ini], n_peaks[-1]],
        s=80,
        color=COLOR5,
        marker="o",
    )

    iters = [
        (iterations[np_ini], f"Half Height t={t[np_ini]:.1f}"),
        (iterations[-1], f"Final State t={t[-1]:.1f}"),
    ]
    for (it, label), ax in zip(iters, [axs[1], axs[2]]):
        cells = crt.load_cells(it, opath)
        ax.set_xlim(0.11 * settings.domain_size, 0.89 * settings.domain_size)
        ax.set_ylim(0.11 * settings.domain_size, 0.89 * settings.domain_size)
        ax.set_axis_off()
        crt.plot_cells(ax, cells, 0.0, conc_max)
        ax.set_title(label)

    fig.subplots_adjust(
        left=0.03, right=0.97, bottom=0.08, top=0.92, wspace=0.01, hspace=0
    )
    fig.savefig("cr_trichome.pdf")
