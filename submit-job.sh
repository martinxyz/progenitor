#!/bin/bash
#
# Run Python experiment-of-the-day on a remote ray cluster.
#
# Compile the Python module on the head node. (The cluster has different arch
# and CPU flags than my workstation.) Then submit it as ray job such that
# parallel jobs can have different modules, as documented in:
# https://docs.ray.io/en/latest/ray-core/handling-dependencies.html#library-development
#
set -e
clusterdir="/home/martin/aws-ray-cluster"
name="$1"
jobdir="${clusterdir}/job_${name}"

if test -f "${jobdir}"; then
    echo "${jobdir} already exists!"
    exit 1
fi

git add -u
git checkout-index -a -f --prefix="${jobdir}/"

(
    git describe --tags
    git rev-parse HEAD >> "${jobdir}/git-version.txt"
    date
) > "${jobdir}/git-version.txt"

cat << EOF > "${jobdir}/start-job.sh"
#!/bin/bash
set -e -x

export RUSTFLAGS="-C target-cpu=native"
maturin develop -m crates/python/Cargo.toml --release
cargo clean

ray job submit \
  --submission-id="${name}" \
  --no-wait --working-dir="." \
  --runtime-env-json='{"py_modules": ["crates/python/progenitor"]}' \
  -- \
  python py/builders_optimize.py ${name}

EOF
chmod +x "${jobdir}/start-job.sh"

echo "Job directory created: ${jobdir}"
echo

cd "${clusterdir}"
./ray rsync-up config.yaml "${jobdir}/" "job_${name}/"
./ray exec config.yaml "cd job_${name}/ && ./start-job.sh"
