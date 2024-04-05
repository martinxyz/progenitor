#!/bin/bash
#
# Run my Python experiment-of-the-day on a remote ray cluster.
#
# Compile the Python module on the head node. (The cluster has different arch
# and CPU flags than my workstation.) Then submit it as ray job such that
# parallel jobs can have different python modules, as documented in:
# https://docs.ray.io/en/latest/ray-core/handling-dependencies.html#library-development
#
set -e
clusterdir="/home/martin/aws-ray-cluster"
name="$1"
jobdir="${clusterdir}/jobs/${name}"
local=false

# if test -d "${jobdir}"; then
#     echo "${jobdir} already exists!"
#     exit 1
# fi
mkdir -p "${jobdir}"

git add -u
git checkout-index -a -f --prefix="${jobdir}/"

(
    git describe --tags
    git rev-parse HEAD >> "${jobdir}/git-version.txt"
    date
) > "${jobdir}/git-version.txt"

# keep it small
rm -rf "${jobdir}/webapp"

if $local; then
    extra_args=''
else
    extra_args='--storage=s3://maxy-ray-experiments/'
fi

cat << EOF > "${jobdir}/start-job.sh"
#!/bin/bash
set -e -x

export RUSTFLAGS="-C target-cpu=native"
maturin develop -m crates/python/Cargo.toml --release
cargo clean

ray job submit \
  --submission-id="${name}" \
  --no-wait \
  --runtime-env-json='{"working_dir": "py/", "py_modules": ["crates/python/progenitor"]}' \
  -- \
  python builders_optimize.py ${name} ${extra_args}
# python ribs_search_hparams.py ${name} ${extra_args}

EOF
chmod +x "${jobdir}/start-job.sh"

echo "Job directory created: ${jobdir}"
echo

cd "${clusterdir}"

if $local; then
    . /home/martin/.cache/pypoetry/virtualenvs/progenitor-experiments-GANURkbN-py3.10/bin/activate
    # ray start --head
    export RAY_ADDRESS='http://127.0.0.1:8265'
    mkdir -p "/tmp/submit-job"
    cp -a "${jobdir}" "/tmp/submit-job/${name}"
    cd "/tmp/submit-job/${name}"
    ./start-job.sh
    ray job logs -f "${name}"
else
    ./ray rsync-up config.yaml "${jobdir}/" "job_${name}/"
    ./ray exec config.yaml "cd job_${name}/ && ./start-job.sh"
fi
