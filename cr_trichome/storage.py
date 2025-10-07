from pathlib import Path
from glob import glob
import numpy as np
import pandas as pd
import json

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
    """
    Loads all cells from a given iteration at the specified output path.

    Parameters
    ----------
    iteration : int
        Iteration number for which to load results.

    output_path : Path | None = None
        Folder of stored results. If not specified,
        we obtain it via the ``get_last_output_path`` function.

    Returns
    -------
    cells : pd.DataFrame
        Flattened dataframe with all values that make up the cell-agent.

    Raises
    ------
    ValueError:
        See ``get_last_output_path``.
    """
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
