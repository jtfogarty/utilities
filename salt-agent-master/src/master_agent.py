# ~/salt-agent-master/src/master_agent.py
import asyncio
from datetime import datetime

from agno.agent import Agent
from agno.models.ollama import Ollama
from agno.tools.mcp import MCPTools

from tools import (
    check_infrastructure,
    list_minions,
    accept_minion,
    run_salt_command,
    apply_salt_state
)

async def run_agent():
    # Connect to local Ollama
    ollama_model = Ollama(id="qwen2.5-coder:latest", host="http://10.10.3.8:11434")

    from mcp.client.stdio import StdioServerParameters
    
    # Connect to SurrealMCP via stdio podman subprocess
    server_params = StdioServerParameters(
        command="podman",
        args=[
            "run", "-i", "--rm", 
            "-e", "SURREALDB_URL=ws://host.containers.internal:8089/rpc",
            "-e", "SURREALDB_NS=salt_agent", 
            "-e", "SURREALDB_DB=master", 
            "-e", "SURREALDB_USER=root", 
            "-e", "SURREALDB_PASS=18b85e792e70178281a2efc29d92e733", 
            "docker.io/surrealdb/surrealmcp:latest", "start", "--auth-disabled"
        ],
        env=None
    )
    
    mcp_tools = MCPTools(server_params=server_params)

    salt_tools = [
        check_infrastructure,
        list_minions,
        accept_minion,
        run_salt_command,
        apply_salt_state
    ]

    # Initialize MCP tools session via async with
    async with mcp_tools:
        # Create the agent
        agent = Agent(
            model=ollama_model,
            tools=[mcp_tools] + salt_tools,
            instructions="""You are the Master Agent for SaltStack infrastructure. 
            Use MCP tools for memory (e.g., Insert or InsertQuery for history, Select or Query for recall).
            For status updates, always call check_infrastructure first before answering to avoid hallucinations.
            Store all actions in SurrealDB under table 'history'.
            """
        )
        
        print("Interactive Master Agent (Powered by Agno & SurrealMCP)\n")

        while True:
            try:
                # Use asyncio run_in_executor to avoid blocking the event loop with input()
                prompt = await asyncio.get_event_loop().run_in_executor(None, input, "\nYou: ")
                prompt = prompt.strip()
                if prompt.lower() in ["exit", "quit", "q"]:
                    break
                if prompt:
                    print("\n=== MASTER AGENT ===\n")
                    
                    response = agent.run(prompt)
                    content = response.content if hasattr(response, 'content') else str(response)

                    import re, json
                    
                    # Check for forced JSON tool call formats (often produced by qwen2.5 local)
                    # Loop to handle consecutive tool calls if the model wants to run multiple
                    while True:
                        tool_call_data = None
                        text_to_parse = content.strip()
                        
                        # Try code block extraction
                        match = re.search(r'```json\s*(\{.*?\})\s*```', text_to_parse, re.DOTALL)
                        if match:
                            try:
                                tool_call_data = json.loads(match.group(1))
                            except json.JSONDecodeError:
                                pass
                        # Try raw JSON block
                        elif text_to_parse.startswith('{') and text_to_parse.endswith('}'):
                            try:
                                tool_call_data = json.loads(text_to_parse)
                            except json.JSONDecodeError:
                                pass
                        
                        # If valid tool call found, execute it
                        if isinstance(tool_call_data, dict) and 'name' in tool_call_data:
                            tool_name = tool_call_data.get('name')
                            tool_args = tool_call_data.get('arguments', {})
                            
                            # Find the python function
                            found_tool = None
                            for t in salt_tools:
                                if t.__name__ == tool_name:
                                    found_tool = t
                                    break
                                    
                            if not found_tool:
                                # Also check MCPTools
                                for t in mcp_tools.get_functions().values():
                                    if getattr(t, 'name', None) == tool_name or getattr(t, '__name__', None) == tool_name:
                                        found_tool = t
                                        break
                                        
                            if found_tool:
                                print(f"\n🔧 Running tool: {tool_name}...")
                                try:
                                    # execute sync or async depending on the tool
                                    import inspect
                                    if inspect.iscoroutinefunction(found_tool) or (hasattr(found_tool, 'execute') and inspect.iscoroutinefunction(found_tool.execute)):
                                        if hasattr(found_tool, 'execute'):
                                            tool_result = await found_tool.execute(**tool_args)
                                        else:
                                            tool_result = await found_tool(**tool_args)
                                    else:
                                        if hasattr(found_tool, 'execute'):
                                            tool_result = found_tool.execute(**tool_args)
                                        else:
                                            tool_result = found_tool(**tool_args)
                                            
                                    print(f"Result: {str(tool_result)[:100]}...\n")
                                except Exception as e:
                                    tool_result = f"Tool Error: {str(e)}"
                                    
                                # Feed the tool execution back into the agent
                                response = agent.run(f"Tool `{tool_name}` output:\n{tool_result}\n\nPlease proceed.")
                                content = response.content if hasattr(response, 'content') else str(response)
                            else:
                                # Break if tool is not found
                                break
                        else:
                            break # Break if no more parsing

                    print(content)
            except KeyboardInterrupt:
                break
            except Exception as e:
                print("Error:", e)

if __name__ == "__main__":
    asyncio.run(run_agent())