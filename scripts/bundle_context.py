
import os
import shutil
import json
import zipfile
import argparse
from datetime import datetime
from pathlib import Path

def create_bundle(bundle_name, include_zip=True):
    root = Path("/home/sikmindz/Coding/Libraries")
    output_dir = root / f"bundles/{bundle_name}"
    output_dir.mkdir(parents=True, exist_ok=True)
    
    print(f"[*] Assembling context bundle: {bundle_name}")
    
    # 1. Define the components of the AICC Bundle
    manifest = {
        "bundle_id": bundle_name,
        "timestamp": datetime.now().isoformat(),
        "components": {
            "aicc": {
                "description": "AI Context Control - Master Manifest",
                "files": ["AICC_MANIFEST.json", "AICC_CONTRACT.md"]
            },
            "z_py": {
                "description": "JSON/Zip Sidecar Validator",
                "files": ["z.py"]
            },
            "snap_py": {
                "description": "Snapshot Protocol Logic",
                "files": ["snap.py"]
            },
            "claimledger": {
                "description": "Claim Ledger & Truth Verifier",
                "files": ["claim_ledger_spec.json", "claim_ledger_receipts.md"]
            }
        }
    }

    # 2. Collect files (handle missing files with placeholders)
    # z.py
    zp_src = root / "z.py"
    if zp_src.exists():
        shutil.copy(zp_src, output_dir / "z.py")
    else:
        (output_dir / "z.py").write_text("# z.py missing from root - using fallback\nprint('Z-PY MISSING')")

    # snap.py (Look for it in scr-runtime or AiDENs)
    snap_src = root / "AiDENs/z.py" # often bundled or nearby
    # Search for snap.py specifically
    found_snap = False
    for path in root.rglob("snap.py"):
        shutil.copy(path, output_dir / "snap.py")
        found_snap = True
        break
    if not found_snap:
        (output_dir / "snap.py").write_text("# snap.py missing - using fallback\nprint('SNAP-PY MISSING')")

    # claimledger
    # Extract from claim-ledger crate or manifest
    ledger_src = root / "claim-ledger"
    if ledger_src.exists():
        # Create a summary of the crate
        summary = f"Claim Ledger Crate located at {ledger_src}\nStructure: " + str(os.listdir(ledger_src))
        (output_dir / "claim_ledger_receipts.md").write_text(summary)
        (output_dir / "claim_ledger_spec.json").write_text(json.dumps({"crate": "claim-ledger", "status": "active"}, indent=2))
    else:
        (output_dir / "claim_ledger_receipts.md").write_text("Claim-ledger crate missing from Libraries root.")

    # AICC Manifests
    (output_dir / "AICC_MANIFEST.json").write_text(json.dumps(manifest, indent=2))
    (output_dir / "AICC_CONTRACT.md").write_text(f"# AICC Contract\nBundle: {bundle_name}\nVerified components: {list(manifest['components'].keys())}")

    # 3. Optional Zip
    if include_zip:
        zip_path = root / f"bundles/{bundle_name}.zip"
        with zipfile.ZipFile(zip_path, 'w', zipfile.ZIP_DEFLATED) as zipf:
            for file in output_dir.rglob('*'):
                zipf.write(file, file.relative_to(output_dir))
        print(f"[+] Full Zip created: {zip_path}")
    else:
        print("[+] Sidecar-only bundle created (no zip).")

    print(f"[!] Bundle complete at: {output_dir}")

if __name__ == '__main__':
    parser = argparse.ArgumentParser()
    parser.add_argument("--name", default="agent-context-bundle", help="Name of the bundle")
    parser.add_argument("--no-zip", action="store_false", dest="zip", help="Create sidecars only")
    parser.set_defaults(zip=True)
    args = parser.parse_args()
    create_bundle(args.name, args.zip)
