#!/usr/bin/env python3
"""
Compare benchmark results with baseline.
Usage: python compare_benchmarks.py --current results.json --baseline baseline.toml --output comparison.json --threshold 0.10
"""

import argparse
import json
import sys
from pathlib import Path

try:
    import tomllib
except ImportError:
    import tomli as tomllib


def parse_benchmark_json(json_path: Path) -> dict:
    """Parse benchmark results JSON."""
    with open(json_path) as f:
        data = json.load(f)
    
    results = {}
    for bench in data.get('benchmarks', []):
        name = bench.get('name', 'unknown')
        results[name] = {
            'mean': bench.get('mean', {}).get('estimate', 0),
            'stddev': bench.get('mean', {}).get('confidence_interval', {}).get('upper_bound', 0) -
                     bench.get('mean', {}).get('confidence_interval', {}).get('lower_bound', 0)
        }
    return results


def parse_baseline_toml(toml_path: Path) -> dict:
    """Parse baseline metrics TOML."""
    with open(toml_path, 'rb') as f:
        data = tomllib.load(f)
    
    results = {}
    for name, values in data.get('benchmarks', {}).items():
        results[name] = {
            'mean': values.get('mean', 0),
            'stddev': values.get('stddev', 0)
        }
    return results


def compare_results(current: dict, baseline: dict, threshold: float) -> dict:
    """Compare current results with baseline."""
    comparison = {
        'improvements': [],
        'regressions': [],
        'unchanged': [],
        'new_benchmarks': [],
        'missing_benchmarks': []
    }
    
    for name, current_data in current.items():
        if name not in baseline:
            comparison['new_benchmarks'].append({
                'name': name,
                'mean': current_data['mean']
            })
            continue
        
        baseline_mean = baseline[name]['mean']
        current_mean = current_data['mean']
        
        if baseline_mean == 0:
            change_pct = 0
        else:
            change_pct = (current_mean - baseline_mean) / baseline_mean
        
        result = {
            'name': name,
            'baseline_mean': baseline_mean,
            'current_mean': current_mean,
            'change_pct': change_pct * 100,
            'status': 'unchanged'
        }
        
        if change_pct > threshold:
            result['status'] = 'regression'
            comparison['regressions'].append(result)
        elif change_pct < -threshold:
            result['status'] = 'improvement'
            comparison['improvements'].append(result)
        else:
            comparison['unchanged'].append(result)
    
    for name in baseline:
        if name not in current:
            comparison['missing_benchmarks'].append({
                'name': name,
                'mean': baseline[name]['mean']
            })
    
    return comparison


def main():
    parser = argparse.ArgumentParser(description='Compare benchmark results')
    parser.add_argument('--current', type=Path, required=True,
                       help='Current benchmark results JSON')
    parser.add_argument('--baseline', type=Path, required=True,
                       help='Baseline metrics TOML')
    parser.add_argument('--output', type=Path, default=Path('comparison-report.json'),
                       help='Output comparison report')
    parser.add_argument('--threshold', type=float, default=0.10,
                       help='Regression threshold (default 10%)')
    args = parser.parse_args()
    
    current = parse_benchmark_json(args.current)
    baseline = parse_baseline_toml(args.baseline)
    
    comparison = compare_results(current, baseline, args.threshold)
    
    report = {
        'threshold': args.threshold,
        'total_benchmarks': len(current),
        'improvements': len(comparison['improvements']),
        'regressions': len(comparison['regressions']),
        'unchanged': len(comparison['unchanged']),
        'details': comparison
    }
    
    with open(args.output, 'w') as f:
        json.dump(report, f, indent=2)
    
    print(f"Benchmark comparison written to {args.output}")
    print(f"Improvements: {report['improvements']}")
    print(f"Regressions: {report['regressions']}")
    print(f"Unchanged: {report['unchanged']}")
    
    if comparison['regressions']:
        print("\nREGRESSIONS DETECTED:")
        for reg in comparison['regressions']:
            print(f"  - {reg['name']}: {reg['change_pct']:.2f}% slower")
        sys.exit(1)
    
    print("\nNo regressions detected!")


if __name__ == '__main__':
    main()
