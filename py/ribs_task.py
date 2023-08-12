import numpy as np

import progenitor
version_check = 13
assert progenitor.mod.version_check == version_check, progenitor.__file__

def param_count():
    Builders = progenitor.mod.Builders
    assert progenitor.mod.version_check == version_check, f'wrong module version: {Builders.__file__}'
    param_count = progenitor.mod.Builders.param_count
    return param_count

def evaluate(x, hyperparams, episodes):
    Builders = progenitor.mod.Builders
    Params = progenitor.mod.Params
    assert progenitor.mod.version_check == version_check, f'wrong module version: {Builders.__file__}'

    score = 0.0
    m_walls_around = 0.0
    m_encounters_log = 0.0
    m_wall_edges = 0.0
    for _ in range(episodes):
        params = Params(x, **hyperparams)
        sim = Builders(params)
        steps = 2000
        sim.steps(steps)
        m_walls_around += sim.walls_nearby / steps  # / n_agents
        m_encounters_log += np.log(sim.encounters + 10)
        m_wall_edges += sim.relative_wall_edges()
        score += sim.hoarding_score

    return {
        "hoarding_score": score / episodes,
        "walls_around": m_walls_around / episodes,
        "wall_edges": m_wall_edges / episodes,
        "encounters": m_encounters_log / episodes,
    }

# QD grid / goal setup
measure_ranges = {
    "walls_around": (0.0, 25.0),
    "wall_edges": (0.7, 1.1),
}
measure_1 = "walls_around"
measure_2 = "wall_edges"
measure_score = "hoarding_score"
