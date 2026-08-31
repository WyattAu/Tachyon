#!/usr/bin/env python3
"""
Check for performance regressions from comparison report.
Usage: python check_regressions.py comparison-report.json
"""

import json
import sys
from pathlib import Path


def main():
    if len(sys.argv) < 2:
        print("Usage: python check_regressions.py comparison-report.json")
        sys.exit(1)
    
    report_path = Path(sys.argv[1])
    
    with open(report_path) as f:
        report = json.load(f)
    
    regressions = report.get('details', {}).get('regressions', [])
    
    if regressions:
        print("PERFORMANCE REGRESSIONS DETECTED:")
        for reg in regressions:
            print(f"  - {reg['name']}: {reg['change_pct']:.2f}% slower")
            print(f"    Baseline: {reg['baseline_mean']:.6f}s")
            print(f"    Current:  {reg['current_mean']:.6f}s")
        print(f"\nTotal regressions: {len(regressions)}")
        sys.exit(1)
    
    print("No performance regressions detected!")
    print(f"Improvements: {report.get('improvements', 0)}")
    print(f"Unchanged: {report.get('unchanged', 0)}")


if __name__ == '__main__':
    main()
