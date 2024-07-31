import matplotlib.pyplot as plt
import matplotlib.patches as mpatches
import matplotlib as mpl
import numpy as np
import json
import pandas as pd
from pathlib import Path
from glob import glob
import tqdm
import multiprocessing as mp


def get_last_output_path(search_path: Path = Path("out/cr_trichome")) -> Path:
    """
    We usually expect a file structure of the form::

        out
        └── cr_trichome
            ├── 2024-07-31-T17-34-27
            ├── 2024-07-31-T17-34-40
            ├── 2024-07-31-T17-34-50
            └── 2024-07-31-T17-34-57

    This function will now obtain the most recent output path.

        >>> get_last_output_path()
        Path("out/cr_trichome/2024-07-31-T17-34-57")

    Parameters
    ----------
    search_path : Path
        The folder in which to search.

    Returns
    -------
    path : Path
        The last simulation path.

    Raises
    ------
    ValueError:
        If `search_path` does not contain any folders.
    """
    folders = sorted(list(glob(str(search_path / "*"))))
    if len(folders) == 0:
        raise ValueError("No folder found in directory {}".format(search_path))
    else:
        return Path(folders[-1])

def get_all_iterations(output_path: Path | None = None) -> np.ndarray:
    """
    Obtain all iterations for the given path.
    Will sort results in ascending order.

    Parameters
    ----------
    output_path : Path
        Folder of stored results. If not specified,
        we obtain it via the ``get_last_output_path`` function.

    Returns
    -------
    iterations : np.ndarray
        Numpy array containing all iterations.

    Raises
    ------
    ValueError:
        See ``get_last_output_path``.
    """
    if output_path is None:
        output_path = get_last_output_path()
    folders = glob(str(output_path / "cells/json/*"))
    return np.sort(np.array([int(Path(f).name) for f in folders]))


def load_cells(iteration: int, output_path: Path | None = None) -> pd.DataFrame:
    if output_path is None:
        output_path = get_last_output_path()

    # Load all json files
    results = []
    for file in glob(str(output_path / "cells/json/{:020}/*.json".format(iteration))):
        f = open(file)
        batch = json.load(f)
        results.extend([b["element"][0] for b in batch["data"]])
    df = pd.json_normalize(results)
    df["cell.mechanics.points"] = df["cell.mechanics.points"].apply(
        lambda x: np.array(x, dtype=float).reshape((2, -1)).T
    )
    df["cell.intracellular"] = df["cell.intracellular"].apply(lambda x: np.array(x, dtype=float))
    return df


def plot_cells(ax, df_cells, intra_low, intra_high):
    viridis = mpl.colormaps['viridis'].resampled(255)
    thresh = intra_low + 0.5 * (intra_high - intra_low)
    for pos, intracellular in zip(df_cells["cell.mechanics.points"], df_cells["cell.intracellular"]):
        if intracellular[2] < thresh:
            c = viridis(0)
        else:
            c = viridis((intracellular[2] - intra_low) / (intra_high - intra_low))
        polygon = mpatches.Polygon(pos, facecolor=c, edgecolor='white')
        ax.add_patch(polygon)


def plot_cells_at_iter(
        iteration: int,
        intra_low: float,
        intra_high: float,
        output_path: Path | None = None,
        save_path: Path | None = None,
        overwrite: bool = False,
        transparent: bool = False,
    ):
    """
    Plot all cells for a given iteration

    Parameters
    ----------
    iteration : int
        Iteration to store. Can be obtained via `get_all_iterations`.
    intra_low : float
        Lower boundary for colorscale of intracellular plotting.
    intra_high : float
        Upper boundary for colorscale of intracellular plotting.
    output_path : Path | None = None
        Path where simulation run is stored.
        If not specified will be given with ``get_last_output_path``.
    save_path : Path | None = None
        Where to store the generated image.
        By default, chooses ``Path(output_path / "images")``.
    overwrite : bool
        Overwrite a filename which is already present with identical name.
    transparent : bool
        Sets the background to transparent if True.

    Raises
    ------
    ValueError:
        See ``get_last_output_path``.
    """
    if output_path is None:
        output_path = get_last_output_path()
    if save_path is None:
        save_path = output_path / "images"
    save_path.mkdir(parents=True, exist_ok=True)
    save_path = save_path / "snapshot-{:020}.png".format(iteration)
    if save_path.exists() and overwrite is False:
        return save_path
    cells = load_cells(iteration, output_path)

    fig, ax = plt.subplots(figsize=(12, 12))
    ax.set_xlim(0, 800)
    ax.set_ylim(0, 800)
    ax.set_axis_off()
    plot_cells(ax, cells, intra_low, intra_high)
    fig.tight_layout()
    fig.savefig(save_path, transparent=transparent)
    plt.close(fig)
    return save_path


def __plotting_helper(args_kwargs):
    args, kwargs = args_kwargs
    return plot_cells_at_iter(*args, **kwargs)


def plot_cells_at_all_iterations(
        intra_min: float,
        intra_max: float,
        output_path: Path | None = None,
        save_path: Path | None = None,
        overwrite: bool = False,
        transparent: bool = True,
    ) -> list:
    if output_path is None:
        output_path = get_last_output_path()
    iterations = get_all_iterations(output_path)
    arguments = [(
        (it, intra_min, intra_max),
        {
            "output_path": output_path,
            "save_path": save_path,
            "overwrite": overwrite,
            "transparent": transparent,
        }
    ) for it in iterations]
    pool = mp.Pool()
    return list(tqdm.tqdm(pool.imap(__plotting_helper, arguments), total=len(iterations)))
