# Reproducible batch replays

## Architecture and scope

The desktop application (`src/main.rs`) owns a winit event loop, camera and egui UI.
`src/app/mod.rs` uploads bodies and an octree to wgpu; `resources/gravity.wgsl`
advances GPU state while rendering. `src/physics/solar_system.rs` initializes the
Sun, eight planets and an asteroid belt. `body.rs` also contains a CPU update
function using the Barnes-Hut tree in `octree.rs`.

There was no replay file format, headless CLI or Python infrastructure. The new
`replay` crate imports these existing physics modules directly, avoiding a second
implementation and the graphics dependencies. Initialization now accepts an RNG
and asteroid count; the desktop wrapper retains its random 10,000-asteroid setup.
The replay backend uses ChaCha8 seeded by an explicit unsigned 64-bit seed, a
fixed timestep and the existing CPU update function.

**These are CPU scenario replays, not recordings of interactive GPU sessions.**
The existing CPU backend uses G=1, theta=1, softening=0.01 and semi-implicit Euler;
the solar-system initializer uses G approximately 39.4784 for orbital velocities.
The GPU shader uses a different integration path. Existing CPU tree limitations
(including coincident bodies at the subdivision limit) remain. This pipeline does
not establish physical accuracy or GPU equivalence.

## Build and run

Requires Python 3.10+ (standard library only) and Rust supporting edition 2024.
From the repository root:

```powershell
cargo build --release --locked --manifest-path replay/Cargo.toml
python batch/run.py --binary replay/target/release/space-replay.exe --output runs/smoke --jobs 6 --seeds 17 29 43 --steps 10 --asteroids 32 --workers 2
```

On Linux/macOS omit `.exe`. `--jobs` is the **total** number of executions across
three distinct seeds, from 3 to 10,000 inclusive. Seeds are assigned round-robin;
10,000 produces 3,334 / 3,333 / 3,333 jobs. Repeated jobs for the same seed replay
the same scenario and should produce identical output. They are repetitions,
not 10,000 distinct initial conditions. Defaults: 3 jobs, seeds 17/29/43, 100
steps, dt=0.001, 10,000 asteroids, up to 4 workers, 300-second timeout per job.
Jobs and bodies are separate counts. Start with the small smoke example.

To request a full batch (this command is an example, not a claim of execution):

```powershell
python batch/run.py --binary replay/target/release/space-replay.exe --output runs/full --jobs 10000 --seeds 17 29 43 --workers 4
```

Each job runs in its own process. At most `--workers` jobs are submitted at once.
Output streams go directly to files, and validation loads at most one job per
worker. Final states for large bodies/batches can consume many GB; provision disk
space accordingly. A timeout kills and waits for the direct replay process.
Ordinary job failures do not stop other jobs. Exit codes: 0 all jobs successful,
1 one or more failed/timed out, 2 invalid configuration or setup/filesystem error.

## Artifacts and recovery

- `manifest.json`: complete ordered plan, numerical settings, executable SHA-256,
  source hashes, Git revision/dirty flag, Python/platform and creation timestamp.
- `jobs/NNNNN/output.json`: validated final bodies (position, velocity,
  acceleration, mass, radius), seed and numerical settings.
- `jobs/NNNNN/stderr.log`: native errors; `stdout.json` retains failed output.
- `jobs/NNNNN/result.json`: status, seed, command, attempt count, exit code,
  UTC start/end, monotonic elapsed seconds, error and successful output hash.
- `results.jsonl`: collected results in completion order (order is not stable).
- `summary.json`: planned, executed, reused, successful, failed and timed-out
  counts, timing and execution settings for the most recent completed invocation.

Use the exact command again with `--resume`. Successful outputs are revalidated
and hashed; missing/corrupt outputs and unsuccessful jobs are rerun. Changed
plans, source hashes, binary or environment reject resume. Worker count and timeout
may change. A fresh run refuses to overwrite an existing run. An exclusive `.lock`
prevents concurrent writers; after a hard kill, verify no runner remains before
removing that lock. Job records are atomically replaced. If interrupted, resume
rebuilds the aggregate journal from job records. Prior attempts' logs are replaced;
archive the directory if every attempt must be retained. A filesystem failure
aborts the runner; already written job records remain recoverable.

Reproduce in a new output directory with the same settings, preserved executable,
source and lockfile, Python version and OS/architecture. Compare output hashes,
not wall-clock metadata or journal ordering. Cross-platform floating point bitwise
identity is not promised. The source hashes describe the checkout at launch;
retain the executable identified by its hash (the runner cannot prove its build
provenance). The lockfile pins replay dependencies independently of the GUI.

## Tests

```powershell
$env:SPACE_REPLAY_BINARY = (Resolve-Path replay/target/release/space-replay.exe).Path
python -m unittest discover -s tests -v
```

Without that variable, integration tests explicitly skip. Tests cover total-plan
limits and seed distribution, failure/nonzero/timeout/malformed-output recording,
real seed repeatability across worker counts, resume reuse, corrupt-output repair,
configuration mismatch rejection and native argument validation. The 10,000-job
plan test constructs metadata only: it does **not** execute 10,000 simulations.
