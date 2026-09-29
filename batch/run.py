"""Bounded, reproducible orchestration for space-replay (Python 3.10+)."""
import argparse
from concurrent.futures import ThreadPoolExecutor, wait, FIRST_COMPLETED
from datetime import datetime, timezone
import hashlib
import json
import math
import os
from pathlib import Path
import platform
import subprocess
import sys
import time


def now():
    return datetime.now(timezone.utc).isoformat()


def digest(path):
    h = hashlib.sha256()
    with Path(path).open('rb') as stream:
        for block in iter(lambda: stream.read(1024 * 1024), b''):
            h.update(block)
    return h.hexdigest()


def save(path, value):
    temporary = path.with_suffix(path.suffix + '.tmp')
    temporary.write_text(json.dumps(value, indent=2, sort_keys=True, allow_nan=False) + '\n', encoding='utf-8')
    temporary.replace(path)


def plan(jobs, seeds):
    if not 3 <= jobs <= 10000:
        raise ValueError('jobs must be between 3 and 10000 total')
    if len(seeds) != 3 or len(set(seeds)) != 3 or any(s < 0 or s >= 2**64 for s in seeds):
        raise ValueError('provide exactly three distinct unsigned 64-bit seeds')
    return [{'id': f'{i:05d}', 'seed': seeds[i % 3]} for i in range(jobs)]


def valid_output(path, job, config):
    data = json.loads(path.read_text(encoding='utf-8'))
    if (data.get('schema_version') != 1 or data.get('backend') != 'cpu-barnes-hut-v1'
            or data.get('seed') != job['seed'] or data.get('steps') != config['steps']
            or data.get('asteroids') != config['asteroids']
            or not math.isclose(data.get('dt', 0), config['dt'], rel_tol=1e-6)
            or len(data.get('bodies', [])) != 9 + config['asteroids']):
        raise ValueError('replay output does not match the job')
    for body in data['bodies']:
        for key in ('position', 'velocity', 'acceleration'):
            if len(body[key]) != 3 or not all(math.isfinite(v) for v in body[key]):
                raise ValueError('invalid body state')
        if not all(math.isfinite(body[k]) and body[k] > 0 for k in ('mass', 'radius')):
            raise ValueError('invalid body properties')


def execute(binary, root, job, config, timeout):
    directory = root / 'jobs' / job['id']
    directory.mkdir(parents=True, exist_ok=True)
    result_path = directory / 'result.json'
    previous = {}
    if result_path.exists():
        try:
            previous = json.loads(result_path.read_text(encoding='utf-8'))
            if (previous['status'] == 'success'
                    and previous['output_sha256'] == digest(directory / 'output.json')):
                valid_output(directory / 'output.json', job, config)
                return previous, True
        except (ValueError, KeyError, OSError, TypeError):
            pass
    command = [str(binary), str(job['seed']), str(config['steps']),
               str(config['dt']), str(config['asteroids'])]
    record = {**job, 'command': command, 'started_at': now(),
              'attempt': previous.get('attempt', 0) + 1, 'status': 'failed', 'returncode': None}
    started = time.perf_counter()
    output = directory / 'stdout.json'
    try:
        with output.open('wb') as out, (directory / 'stderr.log').open('wb') as err:
            process = subprocess.run(command, stdout=out, stderr=err, timeout=timeout, check=False)
        record['returncode'] = process.returncode
        if process.returncode:
            raise RuntimeError(f'replay exited with code {process.returncode}')
        valid_output(output, job, config)
        output.replace(directory / 'output.json')
        record.update(status='success', output_sha256=digest(directory / 'output.json'))
    except subprocess.TimeoutExpired:
        record.update(status='timeout', error=f'exceeded {timeout} seconds')
    except (OSError, ValueError, KeyError, TypeError, RuntimeError) as error:
        record['error'] = f'{type(error).__name__}: {error}'
    record.update(finished_at=now(), duration_seconds=time.perf_counter() - started)
    save(result_path, record)
    return record, False


def source_metadata():
    root = Path(__file__).resolve().parents[1]
    hashes = {}
    for pattern in ('src/physics/*.rs', 'replay/src/*.rs', 'replay/Cargo.*', 'batch/*.py'):
        for path in sorted(root.glob(pattern)):
            hashes[path.relative_to(root).as_posix()] = digest(path)
    try:
        commit = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=root, text=True).strip()
        dirty = bool(subprocess.check_output(['git', 'status', '--porcelain'], cwd=root, text=True).strip())
    except (OSError, subprocess.CalledProcessError):
        commit, dirty = None, None
    return {'commit': commit, 'dirty': dirty, 'source_sha256': hashes}


def run(args):
    jobs = plan(args.jobs, args.seeds)
    if not 1 <= args.workers <= 256 or not math.isfinite(args.timeout) or args.timeout <= 0:
        raise ValueError('workers must be 1..256 and timeout finite and positive')
    if not 1 <= args.steps <= 1000000 or not 0 <= args.asteroids <= 10000:
        raise ValueError('steps must be 1..1000000 and asteroids 0..10000')
    if not math.isfinite(args.dt) or not 1e-38 <= args.dt <= 1e38:
        raise ValueError('dt must be finite, positive and representable as f32')
    binary = args.binary.resolve(strict=True)
    root = args.output.resolve()
    config = {'steps': args.steps, 'dt': args.dt, 'asteroids': args.asteroids}
    identity = {'schema_version': 1, 'config': config, 'jobs': jobs,
                'binary_sha256': digest(binary), 'python': platform.python_version(),
                'platform': platform.platform(), **source_metadata()}
    root.mkdir(parents=True, exist_ok=True)
    lock = root / '.lock'
    # Exclusive creation prevents concurrent writers. A hard-kill may leave this file.
    fd = os.open(lock, os.O_CREAT | os.O_EXCL | os.O_WRONLY)
    os.close(fd)
    try:
        manifest_path = root / 'manifest.json'
        if manifest_path.exists():
            manifest = json.loads(manifest_path.read_text(encoding='utf-8'))
            if not args.resume:
                raise ValueError('run exists; use --resume or a new output directory')
            if manifest['identity'] != identity:
                raise ValueError('resume rejected: plan, executable, source or environment changed')
        else:
            if args.resume or any(p.name != '.lock' for p in root.iterdir()):
                raise ValueError('new run requires an empty output directory without --resume')
            save(manifest_path, {'created_at': now(), 'identity': identity})
        started = time.perf_counter()
        summary = {'started_at': now(), 'planned': len(jobs), 'executed': 0, 'reused': 0,
                   'success': 0, 'failed': 0, 'timeout': 0,
                   'workers': args.workers, 'timeout_seconds': args.timeout}
        with (root / 'results.jsonl').open('w', encoding='utf-8') as journal:
            with ThreadPoolExecutor(max_workers=args.workers) as pool:
                iterator = iter(jobs)
                pending = set()
                def submit():
                    job = next(iterator, None)
                    if job is not None:
                        pending.add(pool.submit(execute, binary, root, job, config, args.timeout))
                for _ in range(min(args.workers, len(jobs))):
                    submit()
                while pending:
                    done, pending = wait(pending, return_when=FIRST_COMPLETED)
                    for future in done:
                        record, reused = future.result()
                        summary['reused' if reused else 'executed'] += 1
                        summary[record['status']] += 1
                        journal.write(json.dumps(record, sort_keys=True) + '\n')
                        journal.flush()
                        submit()
        summary.update(finished_at=now(), duration_seconds=time.perf_counter() - started)
        save(root / 'summary.json', summary)
        print(json.dumps(summary, indent=2))
        return 0 if summary['success'] == len(jobs) else 1
    finally:
        lock.unlink()


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--binary', type=Path, required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--jobs', type=int, default=3, help='total jobs across three seeds (3..10000)')
    parser.add_argument('--seeds', type=int, nargs=3, default=[17, 29, 43])
    parser.add_argument('--steps', type=int, default=100)
    parser.add_argument('--dt', type=float, default=0.001)
    parser.add_argument('--asteroids', type=int, default=10000)
    parser.add_argument('--workers', type=int, default=min(4, os.cpu_count() or 1))
    parser.add_argument('--timeout', type=float, default=300)
    parser.add_argument('--resume', action='store_true')
    args = parser.parse_args()
    try:
        return run(args)
    except (ValueError, OSError) as error:
        parser.exit(2, f'error: {error}\n')


if __name__ == '__main__':
    sys.exit(main())
