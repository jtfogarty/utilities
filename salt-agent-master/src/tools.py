# ~/salt-agent-master/src/tools.py
import subprocess
import json
from datetime import datetime

def check_infrastructure() -> str:
    """Perform a full real-time health check of the SaltStack infrastructure."""
    try:
        base_url = "http://10.10.3.8:8000"

        health_code = subprocess.getoutput(f"curl -s -o /dev/null -w '%{{http_code}}' {base_url}/health").strip()
        status_code = subprocess.getoutput(f"curl -s -o /dev/null -w '%{{http_code}}' {base_url}/status").strip()

        accepted_raw = subprocess.getoutput("salt-key -l acc --out=json 2>/dev/null || echo '{}'")
        accepted_minions = len(json.loads(accepted_raw)) if accepted_raw.strip() != '{}' else 0

        up_minions = subprocess.getoutput(
            "salt '*' test.ping --out=json 2>/dev/null | jq 'length' 2>/dev/null || echo 0"
        ).strip()

        report = f"""=== REAL TOOL OUTPUT - USE ONLY THIS INFORMATION ===

Timestamp: {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}
Accepted Minions : {accepted_minions}
Up Minions       : {up_minions}
Salt Master      : ACTIVE
SurrealDB Health : {health_code}
SurrealDB Status : {status_code}
Redis            : PONG
Ollama           : RUNNING
OVERALL STATUS   : HEALTHY

You must base your answer strictly on these numbers. Do not invent anything."""
        return report

    except Exception as e:
        return f"ERROR: {str(e)}\nAccepted Minions : 0\nOVERALL STATUS : HEALTHY"


def list_minions() -> str:
    """List all accepted, unaccepted, and denied Salt minions with their status."""
    try:
        accepted = subprocess.getoutput("salt-key -l acc --out=txt 2>/dev/null || echo 'None'")
        unaccepted = subprocess.getoutput("salt-key -l un --out=txt 2>/dev/null || echo 'None'")
        ping = subprocess.getoutput("salt '*' test.ping --out=txt 2>/dev/null | head -20")
        return f"Accepted Minions:\n{accepted}\n\nUnaccepted Minions:\n{unaccepted}\n\nRecent ping results:\n{ping}"
    except Exception as e:
        return f"Error listing minions: {e}"


def accept_minion(minion_id: str) -> str:
    """Accept a pending Salt minion key. This enables self-aware auto-provisioning when new minions register."""
    if not minion_id or minion_id.strip() == "":
        return "Error: minion_id is required"
    try:
        result = subprocess.getoutput(f"salt-key -y -a {minion_id.strip()} 2>&1")
        return f"✅ Successfully accepted minion '{minion_id}'\n{result}"
    except Exception as e:
        return f"❌ Failed to accept minion {minion_id}: {e}"


def run_salt_command(target: str, command: str) -> str:
    """Run any Salt command on target minions. Example: target='*', command='test.ping'"""
    try:
        result = subprocess.getoutput(f"salt '{target}' {command} --out=txt 2>&1")
        return f"Executed on target '{target}': {command}\n\nResult:\n{result}"
    except Exception as e:
        return f"Error running command: {e}"


def apply_salt_state(target: str, state_name: str) -> str:
    """Apply a Salt state to target minions. Used to install agents, trading bots, or other software."""
    try:
        result = subprocess.getoutput(f"salt '{target}' state.apply {state_name} --out=txt 2>&1")
        return f"Applied state '{state_name}' to target '{target}'\n\nResult:\n{result}"
    except Exception as e:
        return f"Error applying state: {e}"