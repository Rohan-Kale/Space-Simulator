import argparse
import contextlib
import io
import json
import os
from pathlib import Path
import subprocess
import tempfile
import unittest
from unittest.mock import patch

from batch.run import execute, plan, run, digest


class PipelineTests(unittest.TestCase):
    def test_full_plan_only(self):
        jobs = plan(10000, [17, 29, 43])
        self.assertEqual(len({j['id'] for j in jobs}), 10000)
        self.assertEqual([sum(j['seed'] == s for j in jobs) for s in [17, 29, 43]], [3334, 3333, 3333])
        for count, seeds in [(10001, [1, 2, 3]), (2, [1, 2, 3]), (3, [1, 1, 2]), (3, [-1, 2, 3])]:
            with self.assertRaises(ValueError):
                plan(count, seeds)

    def test_failures_are_recorded(self):
        for outcome, status in [(subprocess.TimeoutExpired('replay', 1), 'timeout'),
                                (OSError('missing executable'), 'failed'),
                                (subprocess.CompletedProcess([], 7), 'failed'),
                                (subprocess.CompletedProcess([], 0), 'failed')]:
            with self.subTest(outcome=outcome), tempfile.TemporaryDirectory() as temp:
                kwargs = {'side_effect': outcome} if isinstance(outcome, Exception) else {'return_value': outcome}
                with patch('batch.run.subprocess.run', **kwargs):
                    result, reused = execute(Path('fake'), Path(temp), {'id': '00000', 'seed': 17},
                                             {'steps': 2, 'dt': .001, 'asteroids': 1}, 1)
                self.assertEqual(result['status'], status)
                self.assertFalse(reused)
                self.assertIn('error', result)
                self.assertGreaterEqual(result['duration_seconds'], 0)
                self.assertTrue((Path(temp) / 'jobs/00000/result.json').exists())


@unittest.skipUnless(os.environ.get('SPACE_REPLAY_BINARY'), 'set SPACE_REPLAY_BINARY for real replay tests')
class ReplayIntegrationTests(unittest.TestCase):
    def test_reproducible_parallel_resume_and_corruption(self):
        with tempfile.TemporaryDirectory() as temp:
            root = Path(temp)
            args = argparse.Namespace(binary=Path(os.environ['SPACE_REPLAY_BINARY']), output=root / 'a',
                                      jobs=6, seeds=[17, 29, 43], steps=4, dt=.001, asteroids=8,
                                      workers=2, timeout=30, resume=False)
            with contextlib.redirect_stdout(io.StringIO()):
                self.assertEqual(run(args), 0)
                first = [digest(root / 'a/jobs' / f'{i:05d}' / 'output.json') for i in range(6)]
                self.assertEqual(first[:3], first[3:])
                self.assertEqual(len(set(first)), 3)
                args.output = root / 'b'
                args.workers = 1
                self.assertEqual(run(args), 0)
                self.assertEqual(first, [digest(root / 'b/jobs' / f'{i:05d}' / 'output.json') for i in range(6)])
                args.resume = True
                self.assertEqual(run(args), 0)
                summary = json.loads((args.output / 'summary.json').read_text())
                self.assertEqual((summary['executed'], summary['reused']), (0, 6))
                (args.output / 'jobs/00000/output.json').write_text('corrupt')
                self.assertEqual(run(args), 0)
                summary = json.loads((args.output / 'summary.json').read_text())
                self.assertEqual((summary['executed'], summary['reused']), (1, 5))
                args.dt = .002
                with self.assertRaisesRegex(ValueError, 'resume rejected'):
                    run(args)

    def test_invalid_native_arguments(self):
        binary = os.environ['SPACE_REPLAY_BINARY']
        for arguments in [[], ['17', '0', '.001', '8'], ['17', '1', 'NaN', '8']]:
            result = subprocess.run([binary, *arguments], capture_output=True)
            self.assertNotEqual(result.returncode, 0)
            self.assertTrue(result.stderr)


if __name__ == '__main__':
    unittest.main()
