import blosc
import pickle
import progenitor
version_check = 16
assert progenitor.mod.version_check == version_check, progenitor.__file__

# We cannot import progenitor.mod.Builders and then use it in the @ray.remote,
# apparently. (I think the @ray.remote object fails to serialize.)
# (Is this still true when starting with the submit-job.sh script?)


def get_params(x, config):
    assert progenitor.mod.version_check == version_check, progenitor.__file__
    Params = progenitor.mod.Params
    Hyperparams = progenitor.mod.Hyperparams
    hp = Hyperparams(**config["rust"])
    if x is None: return hp
    return Params(hp, x)


def save_pik_blosc(filename: str, obj):
    data = pickle.dumps(obj)
    data = blosc.compress(data)
    with open(filename, 'wb') as f:
        f.write(data)


def load_pik_blosc(filename: str):
    with open(filename, 'rb') as f:
        data = f.read()
    data = blosc.decompress(data)
    return pickle.loads(data)


def save_gz(filename: str, data: bytes):
    with open(filename, 'wb') as f:
        f.write(data)
