#!/usr/bin/env python3
"""
Check quality gates and fail if not met.
Usage: python check_quality_gates.py quality-gate-report.json
"""

import json
import sys
from pathlib import Path


def main():
    if len(sys.argv) < 2:
        print("Usage: python check_quality_gates.py quality-gate-report.json")
        sys.exit(1)
    
    report_path = Path(sys.argv[1])
    
    with open(report_path) as f:
        report = json.load(f)
    
    print("=" * 50)
    print("QUALITY GATE CHECK")
    print("=" * 50)
    
    all_passed = True
    
    for check_name, check_data in report.get('checks', {}).items():
        status = check_data.get('status', 'unknown')
        status_icon = '✅' if status == 'pass' else '⚠️' if status == 'skip' else '❌'
        
        print(f"\n{check_name.upper()}: {status_icon} {status}")
        
        for key, value in check_data.items():
            if key != 'status':
                print(f"  - {key}: {value}")
        
        if status == 'fail':
            all_passed = False
    
    print("\n" + "=" * 50)
    
    if all_passed:
        print("QUALITY GATE: ✅ PASSED")
        sys.exit(0)
    else:
        print("QUALITY GATE: ❌ FAILED")
        sys.exit(1)


if __name__ == '__main__':
    main()
