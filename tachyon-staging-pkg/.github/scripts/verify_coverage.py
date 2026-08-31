#!/usr/bin/env python3
"""
Verify code coverage meets minimum thresholds.
Usage: python verify_coverage.py --min-line-coverage 95.0 --min-branch-coverage 90.0
"""

import argparse
import json
import sys
import defusedxml.ElementTree as ET
from pathlib import Path


def parse_cobertura_xml(xml_path: Path) -> dict:
    """Parse Cobertura XML coverage report."""
    tree = ET.parse(xml_path)
    root = tree.getroot()
    
    coverage = root.find('coverage') if root.tag != 'coverage' else root
    
    line_rate = float(coverage.get('line-rate', 0)) * 100
    branch_rate = float(coverage.get('branch-rate', 0)) * 100
    
    return {
        'line_coverage': line_rate,
        'branch_coverage': branch_rate
    }


def parse_codecov_json(json_path: Path) -> dict:
    """Parse Codecov JSON coverage report."""
    with open(json_path) as f:
        data = json.load(f)
    
    return {
        'line_coverage': data.get('totals', {}).get('percent', 0),
        'branch_coverage': data.get('totals', {}).get('branch_percent', 0)
    }


def main():
    parser = argparse.ArgumentParser(description='Verify code coverage thresholds')
    parser.add_argument('--min-line-coverage', type=float, default=80.0,
                       help='Minimum line coverage percentage')
    parser.add_argument('--min-branch-coverage', type=float, default=70.0,
                       help='Minimum branch coverage percentage')
    parser.add_argument('--coverage-dir', type=Path, default=Path('./coverage'),
                       help='Directory containing coverage reports')
    args = parser.parse_args()
    
    # Find coverage report
    coverage_data = None
    
    # Try Cobertura XML
    cobertura_xml = args.coverage_dir / 'cobertura.xml'
    if cobertura_xml.exists():
        coverage_data = parse_cobertura_xml(cobertura_xml)
    
    # Try Codecov JSON
    codecov_json = args.coverage_dir / 'codecov.json'
    if codecov_json.exists() and coverage_data is None:
        coverage_data = parse_codecov_json(codecov_json)
    
    if coverage_data is None:
        print("ERROR: No coverage report found")
        sys.exit(1)
    
    line_cov = coverage_data['line_coverage']
    branch_cov = coverage_data['branch_coverage']
    
    print(f"Line Coverage: {line_cov:.2f}% (minimum: {args.min_line_coverage}%)")
    print(f"Branch Coverage: {branch_cov:.2f}% (minimum: {args.min_branch_coverage}%)")
    
    errors = []
    
    if line_cov < args.min_line_coverage:
        errors.append(f"Line coverage {line_cov:.2f}% is below minimum {args.min_line_coverage}%")
    
    if branch_cov < args.min_branch_coverage:
        errors.append(f"Branch coverage {branch_cov:.2f}% is below minimum {args.min_branch_coverage}%")
    
    if errors:
        print("\nCoverage check FAILED:")
        for error in errors:
            print(f"  - {error}")
        sys.exit(1)
    
    print("\nCoverage check PASSED!")


if __name__ == '__main__':
    main()
