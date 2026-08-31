#!/usr/bin/env python3
"""
Generate security report from security scan results.
Usage: python generate_security_report.py --output security-report.json
"""

import argparse
import json
import subprocess
import sys
from datetime import datetime
from pathlib import Path


def run_cargo_audit() -> dict:
    """Run cargo audit and parse results."""
    try:
        result = subprocess.run(
            ['cargo', 'audit', '--output', 'json'],
            capture_output=True,
            text=True,
            timeout=300
        )
        if result.returncode == 0:
            return {'status': 'pass', 'vulnerabilities': []}
        
        # Parse JSON output if available
        try:
            data = json.loads(result.stdout)
            return {
                'status': 'fail' if data.get('vulnerabilities', {}).get('list') else 'pass',
                'vulnerabilities': data.get('vulnerabilities', {}).get('list', [])
            }
        except json.JSONDecodeError:
            return {'status': 'error', 'message': result.stderr}
    except Exception as e:
        return {'status': 'error', 'message': str(e)}


def run_cargo_deny() -> dict:
    """Run cargo deny checks."""
    try:
        result = subprocess.run(
            ['cargo', 'deny', 'check', '--output-format', 'json'],
            capture_output=True,
            text=True,
            timeout=300
        )
        
        if result.returncode == 0:
            return {'status': 'pass', 'issues': []}
        
        try:
            data = json.loads(result.stdout)
            return {
                'status': 'fail',
                'issues': data.get('issues', [])
            }
        except json.JSONDecodeError:
            return {'status': 'error', 'message': result.stderr}
    except Exception as e:
        return {'status': 'error', 'message': str(e)}


def check_secrets() -> dict:
    """Check for secrets in the codebase."""
    try:
        result = subprocess.run(
            ['gitleaks', 'detect', '--source', '.', '--no-git', '-f', 'json'],
            capture_output=True,
            text=True,
            timeout=300
        )
        
        if result.returncode == 0:
            return {'status': 'pass', 'leaks': []}
        
        try:
            leaks = json.loads(result.stdout)
            return {
                'status': 'fail' if leaks else 'pass',
                'leaks': leaks
            }
        except json.JSONDecodeError:
            return {'status': 'pass', 'leaks': []}
    except Exception as e:
        return {'status': 'error', 'message': str(e)}


def main():
    parser = argparse.ArgumentParser(description='Generate security report')
    parser.add_argument('--output', type=Path, default=Path('security-report.json'),
                       help='Output file path')
    args = parser.parse_args()
    
    report = {
        'timestamp': datetime.utcnow().isoformat(),
        'checks': {
            'cargo_audit': run_cargo_audit(),
            'cargo_deny': run_cargo_deny(),
            'secrets_scan': check_secrets()
        },
        'summary': {
            'total_checks': 3,
            'passed': 0,
            'failed': 0,
            'errors': 0
        }
    }
    
    # Calculate summary
    for check in report['checks'].values():
        status = check.get('status', 'error')
        if status == 'pass':
            report['summary']['passed'] += 1
        elif status == 'fail':
            report['summary']['failed'] += 1
        else:
            report['summary']['errors'] += 1
    
    # Write report
    with open(args.output, 'w') as f:
        json.dump(report, f, indent=2)
    
    print(f"Security report written to {args.output}")
    print(f"Summary: {report['summary']['passed']} passed, {report['summary']['failed']} failed, {report['summary']['errors']} errors")
    
    # Exit with error if any checks failed
    if report['summary']['failed'] > 0:
        sys.exit(1)


if __name__ == '__main__':
    main()
