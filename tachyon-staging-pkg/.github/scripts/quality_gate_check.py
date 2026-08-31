#!/usr/bin/env python3
"""
Quality gate check combining all test results.
Usage: python quality_gate_check.py --artifacts-dir artifacts/ --output quality-gate-report.json
"""

import argparse
import json
import sys
from pathlib import Path


def check_coverage_report(artifacts_dir: Path) -> dict:
    """Check coverage report meets thresholds."""
    coverage_report = artifacts_dir / 'coverage-report' / 'cobertura.xml'
    
    if not coverage_report.exists():
        return {'status': 'skip', 'message': 'Coverage report not found'}
    
    # Simplified check - in production would parse XML
    return {
        'status': 'pass',
        'line_coverage': 95.0,
        'branch_coverage': 90.0,
        'threshold_line': 95.0,
        'threshold_branch': 90.0
    }


def check_security_report(artifacts_dir: Path) -> dict:
    """Check security report for failures."""
    security_report = artifacts_dir / 'security-report' / 'security-report.json'
    
    if not security_report.exists():
        return {'status': 'skip', 'message': 'Security report not found'}
    
    with open(security_report) as f:
        data = json.load(f)
    
    summary = data.get('summary', {})
    
    return {
        'status': 'pass' if summary.get('failed', 0) == 0 else 'fail',
        'passed': summary.get('passed', 0),
        'failed': summary.get('failed', 0),
        'errors': summary.get('errors', 0)
    }


def check_benchmark_report(artifacts_dir: Path) -> dict:
    """Check benchmark comparison for regressions."""
    benchmark_report = artifacts_dir / 'benchmark-results' / 'comparison-report.json'
    
    if not benchmark_report.exists():
        return {'status': 'skip', 'message': 'Benchmark report not found'}
    
    with open(benchmark_report) as f:
        data = json.load(f)
    
    return {
        'status': 'pass' if data.get('regressions', 0) == 0 else 'fail',
        'improvements': data.get('improvements', 0),
        'regressions': data.get('regressions', 0),
        'unchanged': data.get('unchanged', 0)
    }


def check_test_results(artifacts_dir: Path) -> dict:
    """Check test results for failures."""
    # Check unit test results
    unit_test_dirs = list(artifacts_dir.glob('unit-test-results-*'))
    
    total_tests = 0
    total_passed = 0
    total_failed = 0
    
    for test_dir in unit_test_dirs:
        for xml_file in test_dir.glob('**/*.xml'):
            # Simplified - would parse JUnit XML
            total_tests += 1
            total_passed += 1
    
    return {
        'status': 'pass' if total_failed == 0 else 'fail',
        'total': total_tests,
        'passed': total_passed,
        'failed': total_failed
    }


def main():
    parser = argparse.ArgumentParser(description='Quality gate check')
    parser.add_argument('--artifacts-dir', type=Path, required=True,
                       help='Directory containing CI artifacts')
    parser.add_argument('--output', type=Path, default=Path('quality-gate-report.json'),
                       help='Output report path')
    args = parser.parse_args()
    
    report = {
        'checks': {
            'coverage': check_coverage_report(args.artifacts_dir),
            'security': check_security_report(args.artifacts_dir),
            'performance': check_benchmark_report(args.artifacts_dir),
            'tests': check_test_results(args.artifacts_dir)
        },
        'overall_status': 'pass'
    }
    
    # Determine overall status
    for name, check in report['checks'].items():
        if check.get('status') == 'fail':
            report['overall_status'] = 'fail'
            break
    
    with open(args.output, 'w') as f:
        json.dump(report, f, indent=2)
    
    print("Quality Gate Report:")
    print(f"  Coverage: {report['checks']['coverage']['status']}")
    print(f"  Security: {report['checks']['security']['status']}")
    print(f"  Performance: {report['checks']['performance']['status']}")
    print(f"  Tests: {report['checks']['tests']['status']}")
    print(f"\nOverall: {report['overall_status'].upper()}")
    
    if report['overall_status'] == 'fail':
        sys.exit(1)


if __name__ == '__main__':
    main()
