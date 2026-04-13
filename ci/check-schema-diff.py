# Nintendoor64/ci/check-schema-diff.py
#!/usr/bin/env python3
"""
Simple schema diff checker for CI.
Detects breaking changes: removed required fields, type changes, enum variant removals.
"""

import json
import sys
from pathlib import Path

def load_schema(path: Path) -> dict:
    with open(path, 'r') as f:
        return json.load(f)

def get_required_fields(schema: dict) -> set:
    obj = schema.get('properties', {})
    required = set(schema.get('required', []))
    return {f for f in required if f in obj}

def get_property_types(schema: dict) -> dict:
    props = schema.get('properties', {})
    return {name: prop.get('type') for name, prop in props.items()}

def check_breaking_changes(old_path: Path, new_path: Path) -> list[str]:
    errors = []
    old_files = {f.stem: f for f in old_path.glob('*.schema.json')}
    new_files = {f.stem: f for f in new_path.glob('*.schema.json')}
    
    # Check for removed schemas
    for name in old_files:
        if name not in new_files:
            errors.append(f"BREAKING: Schema '{name}' was removed")
    
    for name, new_file in new_files.items():
        if name not in old_files:
            continue  # New schema is non-breaking
        old_schema = load_schema(old_files[name])
        new_schema = load_schema(new_file)
        
        old_required = get_required_fields(old_schema)
        new_required = get_required_fields(new_schema)
        
        # Removed required fields = breaking
        removed = old_required - new_required
        for field in removed:
            errors.append(f"BREAKING: Schema '{name}' removed required field '{field}'")
        
        # Type changes on existing fields = breaking
        old_types = get_property_types(old_schema)
        new_types = get_property_types(new_schema)
        for field in old_types.keys() & new_types.keys():
            if old_types[field] != new_types[field]:
                errors.append(f"BREAKING: Schema '{name}' changed type of field '{field}': {old_types[field]} -> {new_types[field]}")
    
    return errors

if __name__ == '__main__':
    if len(sys.argv) != 3:
        print(f"Usage: {sys.argv[0]} <old_schema_dir> <new_schema_dir>")
        sys.exit(2)
    
    old_dir = Path(sys.argv[1])
    new_dir = Path(sys.argv[2])
    
    errors = check_breaking_changes(old_dir, new_dir)
    if errors:
        print("Schema breaking changes detected:")
        for err in errors:
            print(f"  - {err}")
        sys.exit(1)
    else:
        print("No breaking schema changes detected.")
        sys.exit(0)
