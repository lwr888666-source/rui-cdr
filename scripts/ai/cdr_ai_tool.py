"""
rui-ColeSDK - AI Integration & MCP Tool
Allows AI Agents (Antigravity, Claude Desktop, Cursor, OpenAI, etc.) to inspect and convert .cdr files.
"""

import sys
import os
import json
import subprocess
from pathlib import Path

def find_cli_binary():
    """Finds rui-cdr-cli binary in PATH or nearby release/debug folders."""
    candidates = [
        "rui-cdr-cli.exe",
        "rui-cdr-cli",
        "tracedraw-cli.exe",
        "tracedraw-cli",
        str(Path(__file__).resolve().parent.parent.parent / "target" / "release" / "rui-cdr-cli.exe"),
        str(Path(__file__).resolve().parent.parent.parent / "target" / "release" / "tracedraw-cli.exe"),
        str(Path(__file__).resolve().parent.parent.parent / "dist" / "rui-cdr-cli.exe"),
    ]
    for c in candidates:
        if os.path.exists(c):
            return str(Path(c).resolve())
        # Check PATH
        which = subprocess.run(["where.exe" if os.name == "nt" else "which", c],
                               capture_output=True, text=True)
        if which.returncode == 0 and which.stdout.strip():
            return which.stdout.strip().splitlines()[0]
    return "rui-cdr-cli"

def get_cdr_info(file_path: str) -> dict:
    """Extracts metadata, object hierarchy, text and page dimensions from a .cdr file."""
    cli = find_cli_binary()
    res = subprocess.run([cli, "info", file_path], capture_output=True, text=True, encoding="utf-8", errors="replace")
    if res.returncode != 0:
        return {"error": res.stderr.strip() or "Failed to inspect file"}
    return {
        "file": file_path,
        "raw_info": res.stdout.strip()
    }

def convert_cdr(in_path: str, out_path: str) -> dict:
    """Converts a .cdr file to svg, png, pdf, eps, dxf, emf, etc."""
    cli = find_cli_binary()
    res = subprocess.run([cli, "convert", in_path, out_path], capture_output=True, text=True, encoding="utf-8", errors="replace")
    if res.returncode != 0:
        return {"success": False, "error": res.stderr.strip() or "Conversion failed"}
    return {
        "success": True,
        "input": in_path,
        "output": out_path,
        "size_bytes": os.path.getsize(out_path) if os.path.exists(out_path) else 0
    }

if __name__ == "__main__":
    if len(sys.argv) < 2:
        print("Usage:")
        print("  python cdr_ai_tool.py info <file.cdr>")
        print("  python cdr_ai_tool.py convert <input.cdr> <output.png|svg|pdf>")
        sys.exit(1)

    cmd = sys.argv[1].lower()
    if cmd == "info" and len(sys.argv) >= 3:
        info = get_cdr_info(sys.argv[2])
        print(json.dumps(info, ensure_ascii=False, indent=2))
    elif cmd == "convert" and len(sys.argv) >= 4:
        ret = convert_cdr(sys.argv[2], sys.argv[3])
        print(json.dumps(ret, ensure_ascii=False, indent=2))
    else:
        print("Invalid arguments. Run with --help for usage.")
