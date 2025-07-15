#!/usr/bin/env python3

"""
CE-DPS Phase Validator
Validates completion of each phase in the CE-DPS methodology
"""

import json
import os
import sys
import argparse
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Optional, Tuple
import subprocess

class PhaseValidator:
    def __init__(self, project_path: str):
        self.project_path = Path(project_path)
        self.results = {
            "timestamp": datetime.utcnow().isoformat(),
            "project_path": str(self.project_path),
            "validation_results": {}
        }
    
    def validate_phase_1(self) -> Dict:
        """Validate Phase 1: Strategic Planning completion"""
        print("🔍 Validating Phase 1: Strategic Planning")
        
        checks = {
            "business_requirements": self._check_business_requirements(),
            "architecture_approved": self._check_architecture_approval(),
            "roadmap_created": self._check_roadmap_creation(),
            "risk_assessment": self._check_risk_assessment(),
            "human_signoff": self._check_human_signoff("phase-1")
        }
        
        all_passed = all(checks.values())
        
        return {
            "phase": "Phase 1: Strategic Planning",
            "status": "PASSED" if all_passed else "FAILED",
            "checks": checks,
            "required_for_phase_2": all_passed
        }
    
    def validate_phase_2(self) -> Dict:
        """Validate Phase 2: Sprint Planning completion"""
        print("🔍 Validating Phase 2: Sprint Planning")
        
        checks = {
            "features_selected": self._check_feature_selection(),
            "implementation_plan": self._check_implementation_plan(),
            "complexity_assessed": self._check_complexity_assessment(),
            "dependencies_identified": self._check_dependencies(),
            "timeline_approved": self._check_timeline_approval(),
            "human_signoff": self._check_human_signoff("phase-2")
        }
        
        all_passed = all(checks.values())
        
        return {
            "phase": "Phase 2: Sprint Planning",
            "status": "PASSED" if all_passed else "FAILED",
            "checks": checks,
            "required_for_phase_3": all_passed
        }
    
    def validate_phase_3(self) -> Dict:
        """Validate Phase 3: Implementation completion"""
        print("🔍 Validating Phase 3: Implementation")
        
        checks = {
            "code_implemented": self._check_code_implementation(),
            "tests_comprehensive": self._check_comprehensive_testing(),
            "quality_gates_passed": self._check_quality_gates(),
            "documentation_complete": self._check_documentation(),
            "security_validated": self._check_security_validation(),
            "performance_tested": self._check_performance_testing(),
            "human_validation": self._check_human_business_validation()
        }
        
        all_passed = all(checks.values())
        
        return {
            "phase": "Phase 3: Implementation",
            "status": "PASSED" if all_passed else "FAILED",
            "checks": checks,
            "ready_for_production": all_passed
        }
    
    def _check_business_requirements(self) -> bool:
        """Check if business requirements are documented"""
        req_files = [
            "docs/requirements.md",
            "docs/business-requirements.md",
            "README.md"
        ]
        
        for file_path in req_files:
            full_path = self.project_path / file_path
            if full_path.exists():
                content = full_path.read_text()
                if any(keyword in content.lower() for keyword in [
                    "business", "requirements", "success metrics", "target users"
                ]):
                    return True
        return False
    
    def _check_architecture_approval(self) -> bool:
        """Check if architecture has been approved"""
        arch_files = [
            "docs/architecture.md",
            "docs/design.md",
            "docs/system-design.md"
        ]
        
        for file_path in arch_files:
            full_path = self.project_path / file_path
            if full_path.exists():
                content = full_path.read_text()
                if any(keyword in content.lower() for keyword in [
                    "approved", "architecture", "system design", "approved by"
                ]):
                    return True
        return False
    
    def _check_roadmap_creation(self) -> bool:
        """Check if feature roadmap exists"""
        roadmap_files = [
            "docs/roadmap.md",
            "docs/features.md",
            "docs/sprint-plan.md"
        ]
        
        for file_path in roadmap_files:
            full_path = self.project_path / file_path
            if full_path.exists():
                content = full_path.read_text()
                if any(keyword in content.lower() for keyword in [
                    "roadmap", "features", "sprint", "milestone"
                ]):
                    return True
        return False
    
    def _check_risk_assessment(self) -> bool:
        """Check if risk assessment is documented"""
        risk_files = [
            "docs/risks.md",
            "docs/risk-assessment.md",
            "docs/architecture.md"
        ]
        
        for file_path in risk_files:
            full_path = self.project_path / file_path
            if full_path.exists():
                content = full_path.read_text()
                if any(keyword in content.lower() for keyword in [
                    "risk", "mitigation", "threat", "vulnerability"
                ]):
                    return True
        return False
    
    def _check_human_signoff(self, phase: str) -> bool:
        """Check if human signoff exists for phase"""
        signoff_files = [
            f"docs/{phase}-signoff.md",
            f"docs/{phase}-approval.md",
            "docs/approvals.md"
        ]
        
        for file_path in signoff_files:
            full_path = self.project_path / file_path
            if full_path.exists():
                content = full_path.read_text()
                if any(keyword in content.lower() for keyword in [
                    "approved", "signoff", "sign-off", "authorized"
                ]):
                    return True
        return False
    
    def _check_feature_selection(self) -> bool:
        """Check if features are selected for sprint"""
        feature_files = [
            "docs/sprint-features.md",
            "docs/backlog.md",
            "docs/features.md"
        ]
        
        for file_path in feature_files:
            full_path = self.project_path / file_path
            if full_path.exists():
                content = full_path.read_text()
                if any(keyword in content.lower() for keyword in [
                    "selected", "sprint", "features", "priority"
                ]):
                    return True
        return False
    
    def _check_implementation_plan(self) -> bool:
        """Check if implementation plan exists"""
        plan_files = [
            "docs/implementation-plan.md",
            "docs/sprint-plan.md",
            "docs/development-plan.md"
        ]
        
        for file_path in plan_files:
            full_path = self.project_path / file_path
            if full_path.exists():
                content = full_path.read_text()
                if any(keyword in content.lower() for keyword in [
                    "implementation", "plan", "tasks", "timeline"
                ]):
                    return True
        return False
    
    def _check_complexity_assessment(self) -> bool:
        """Check if complexity assessment is done"""
        assessment_files = [
            "docs/complexity-assessment.md",
            "docs/effort-estimation.md",
            "docs/sprint-plan.md"
        ]
        
        for file_path in assessment_files:
            full_path = self.project_path / file_path
            if full_path.exists():
                content = full_path.read_text()
                if any(keyword in content.lower() for keyword in [
                    "complexity", "effort", "estimation", "hours"
                ]):
                    return True
        return False
    
    def _check_dependencies(self) -> bool:
        """Check if dependencies are identified"""
        dep_files = [
            "Cargo.toml",
            "package.json",
            "requirements.txt",
            "docs/dependencies.md"
        ]
        
        for file_path in dep_files:
            full_path = self.project_path / file_path
            if full_path.exists():
                return True
        return False
    
    def _check_timeline_approval(self) -> bool:
        """Check if timeline is approved"""
        timeline_files = [
            "docs/timeline.md",
            "docs/sprint-plan.md",
            "docs/schedule.md"
        ]
        
        for file_path in timeline_files:
            full_path = self.project_path / file_path
            if full_path.exists():
                content = full_path.read_text()
                if any(keyword in content.lower() for keyword in [
                    "approved", "timeline", "schedule", "deadline"
                ]):
                    return True
        return False
    
    def _check_code_implementation(self) -> bool:
        """Check if code is implemented"""
        src_dirs = ["src", "lib", "app"]
        
        for src_dir in src_dirs:
            src_path = self.project_path / src_dir
            if src_path.exists() and any(src_path.iterdir()):
                return True
        return False
    
    def _check_comprehensive_testing(self) -> bool:
        """Check if comprehensive tests exist"""
        test_dirs = ["tests", "test"]
        test_files = ["src/lib.rs", "src/main.rs"]
        
        # Check for test directories
        for test_dir in test_dirs:
            test_path = self.project_path / test_dir
            if test_path.exists() and any(test_path.iterdir()):
                return True
        
        # Check for test modules in source files
        for test_file in test_files:
            test_path = self.project_path / test_file
            if test_path.exists():
                content = test_path.read_text()
                if "#[test]" in content or "#[cfg(test)]" in content:
                    return True
        
        return False
    
    def _check_quality_gates(self) -> bool:
        """Check if quality gates have been run"""
        quality_files = [
            "target/quality-report.json",
            "target/coverage/cobertura.xml",
            "target/criterion"
        ]
        
        for file_path in quality_files:
            full_path = self.project_path / file_path
            if full_path.exists():
                return True
        
        # Check if quality gates script was run recently
        try:
            result = subprocess.run(
                ["git", "log", "--oneline", "-10"],
                cwd=self.project_path,
                capture_output=True,
                text=True
            )
            if "quality" in result.stdout.lower():
                return True
        except:
            pass
        
        return False
    
    def _check_documentation(self) -> bool:
        """Check if documentation is complete"""
        doc_files = [
            "README.md",
            "docs/api.md",
            "docs/usage.md",
            "target/doc"
        ]
        
        for file_path in doc_files:
            full_path = self.project_path / file_path
            if full_path.exists():
                if full_path.is_file():
                    content = full_path.read_text()
                    if len(content) > 500:  # Substantial documentation
                        return True
                elif full_path.is_dir() and any(full_path.iterdir()):
                    return True
        return False
    
    def _check_security_validation(self) -> bool:
        """Check if security validation is done"""
        security_indicators = [
            "cargo audit",
            "security scan",
            "vulnerability assessment"
        ]
        
        # Check git history for security-related commits
        try:
            result = subprocess.run(
                ["git", "log", "--grep=security", "--oneline", "-5"],
                cwd=self.project_path,
                capture_output=True,
                text=True
            )
            if result.stdout.strip():
                return True
        except:
            pass
        
        # Check for security-related files
        security_files = [
            "docs/security.md",
            "security-report.json",
            "audit-report.json"
        ]
        
        for file_path in security_files:
            full_path = self.project_path / file_path
            if full_path.exists():
                return True
        
        return False
    
    def _check_performance_testing(self) -> bool:
        """Check if performance testing is done"""
        perf_indicators = [
            "benches",
            "benchmark",
            "performance",
            "load test"
        ]
        
        # Check for benchmark directory
        bench_path = self.project_path / "benches"
        if bench_path.exists() and any(bench_path.iterdir()):
            return True
        
        # Check for performance-related files
        perf_files = [
            "docs/performance.md",
            "performance-report.json",
            "target/criterion"
        ]
        
        for file_path in perf_files:
            full_path = self.project_path / file_path
            if full_path.exists():
                return True
        
        return False
    
    def _check_human_business_validation(self) -> bool:
        """Check if human business validation is complete"""
        validation_files = [
            "docs/business-validation.md",
            "docs/feature-validation.md",
            "docs/user-acceptance.md"
        ]
        
        for file_path in validation_files:
            full_path = self.project_path / file_path
            if full_path.exists():
                content = full_path.read_text()
                if any(keyword in content.lower() for keyword in [
                    "validated", "approved", "tested", "accepted"
                ]):
                    return True
        return False
    
    def generate_report(self, phase: Optional[str] = None) -> Dict:
        """Generate validation report for specified phase or all phases"""
        if phase == "1":
            self.results["validation_results"]["phase_1"] = self.validate_phase_1()
        elif phase == "2":
            self.results["validation_results"]["phase_2"] = self.validate_phase_2()
        elif phase == "3":
            self.results["validation_results"]["phase_3"] = self.validate_phase_3()
        else:
            # Validate all phases
            self.results["validation_results"]["phase_1"] = self.validate_phase_1()
            self.results["validation_results"]["phase_2"] = self.validate_phase_2()
            self.results["validation_results"]["phase_3"] = self.validate_phase_3()
        
        return self.results
    
    def print_summary(self):
        """Print validation summary"""
        print("\n" + "="*60)
        print("🎯 CE-DPS Phase Validation Summary")
        print("="*60)
        
        for phase_key, phase_result in self.results["validation_results"].items():
            status_color = "🟢" if phase_result["status"] == "PASSED" else "🔴"
            print(f"{status_color} {phase_result['phase']}: {phase_result['status']}")
            
            failed_checks = [k for k, v in phase_result["checks"].items() if not v]
            if failed_checks:
                print(f"   Failed checks: {', '.join(failed_checks)}")
        
        print("\n" + "="*60)

def main():
    parser = argparse.ArgumentParser(description='CE-DPS Phase Validator')
    parser.add_argument('--phase', choices=['1', '2', '3'], 
                       help='Validate specific phase (1=Planning, 2=Sprint, 3=Implementation)')
    parser.add_argument('--project-path', default='.', 
                       help='Path to project directory')
    parser.add_argument('--output', help='Output file for JSON report')
    
    args = parser.parse_args()
    
    validator = PhaseValidator(args.project_path)
    results = validator.generate_report(args.phase)
    
    if args.output:
        with open(args.output, 'w') as f:
            json.dump(results, f, indent=2)
        print(f"📄 Report saved to {args.output}")
    
    validator.print_summary()
    
    # Exit with error code if any phase failed
    failed_phases = [p for p in results["validation_results"].values() 
                    if p["status"] == "FAILED"]
    sys.exit(1 if failed_phases else 0)

if __name__ == "__main__":
    main()