<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { openUrl as openUrlInBrowser } from "@tauri-apps/plugin-opener";
  import { onMount, onDestroy } from "svelte";

  interface PidInfo {
    pid: number;
    ports: string;
    user: string;
    cpu: string;
    mem: string;
  }

  interface PortInfo {
    process_name: string;
    command: string;
    pids: PidInfo[];
  }

  let ports = $state<PortInfo[]>([]);
  let error = $state("");
  let loading = $state(true);
  let interval: number;
  let autoRefresh = $state(true);

  async function loadPorts() {
    try {
      error = "";
      ports = await invoke<PortInfo[]>("list_ports");
      loading = false;
    } catch (e) {
      error = `Error loading ports: ${e}`;
      loading = false;
    }
  }

  function getPortUrl(portStr: string): string | null {
    const portNum = parseInt(portStr);
    if (isNaN(portNum)) return null;

    // Common HTTP/HTTPS ports
    if (portNum === 80 || portNum === 8080 || portNum === 3000 || portNum === 4200 ||
        portNum === 5000 || portNum === 8000 || portNum === 9000 || portNum === 3001 ||
        (portNum >= 1024 && portNum <= 65535)) {
      return `http://localhost:${portNum}`;
    }

    return null;
  }

  async function openUrl(url: string) {
    try {
      await openUrlInBrowser(url);
    } catch (e) {
      console.error('Failed to open URL:', e);
      // Fallback to window.open
      window.open(url, '_blank');
    }
  }

  async function killProcess(pid: number) {
    console.debug(`[Frontend] Kill button clicked for PID: ${pid}`);
    console.debug(`[Frontend] Invoking kill_process with PID: ${pid}`);

    try {
      const result = await invoke("kill_process", { pid });
      console.debug(`[Frontend] Kill result:`, result);

      // Wait a bit for the process to actually die
      setTimeout(async () => {
        console.debug(`[Frontend] Refreshing port list after kill`);
        await loadPorts();
      }, 300);
    } catch (e) {
      console.error(`[Frontend] Error killing process:`, e);
      error = `Error killing process: ${e}`;
      setTimeout(() => { error = ""; }, 3000);
    }
  }

  function toggleAutoRefresh() {
    autoRefresh = !autoRefresh;
    if (autoRefresh) {
      interval = setInterval(loadPorts, 5000);
    } else {
      if (interval) {
        clearInterval(interval);
      }
    }
  }

  onMount(() => {
    loadPorts();
    // Auto-refresh every 5 seconds (less aggressive)
    interval = setInterval(loadPorts, 5000);
  });

  onDestroy(() => {
    if (interval) {
      clearInterval(interval);
    }
  });
</script>

<main>
  <div class="drag-region" data-tauri-drag-region></div>
  <div class="container">
    <div class="header">
      <h1>Process Monitor</h1>
      <div class="controls">
        <button class="refresh-btn" onclick={loadPorts}>Refresh Now</button>
        <button class="toggle-btn" onclick={toggleAutoRefresh}>
          {autoRefresh ? "Disable" : "Enable"} Auto-Refresh
        </button>
      </div>
    </div>

    {#if error}
      <div class="error">{error}</div>
    {/if}

    {#if loading}
      <p>Loading ports...</p>
    {:else if ports.length === 0}
      <p>No open ports detected</p>
    {:else}
      <div class="cards">
        {#each ports as processGroup}
          <div class="card">
            <div class="card-header">
              <div class="process-info">
                <span class="process-name">{processGroup.process_name}</span>
                <div class="command-wrapper">
                  <div class="command-path">{processGroup.command || 'N/A'}</div>
                </div>
              </div>
            </div>
            <div class="card-body">
              {#each processGroup.pids as pidInfo}
                <div class="pid-row">
                  <div class="pid-info">
                    <div class="pid-label">PID</div>
                    <div class="pid-value">{pidInfo.pid}</div>
                  </div>
                  <div class="ports-info">
                    <div class="port-label">Ports</div>
                    <div class="port-values">
                      {#each pidInfo.ports.split(', ') as singlePort}
                        {#if getPortUrl(singlePort)}
                          <button
                            type="button"
                            class="port-badge clickable"
                            onclick={(e) => {
                              e.stopPropagation();
                              openUrl(getPortUrl(singlePort)!);
                            }}
                            title="Open http://localhost:{singlePort}"
                          >
                            {singlePort}
                          </button>
                        {:else}
                          <span class="port-badge">
                            {singlePort}
                          </span>
                        {/if}
                      {/each}
                    </div>
                  </div>
                  <div class="stats-info">
                    <div class="stat">
                      <span class="stat-label">User</span>
                      <span class="stat-value">{pidInfo.user}</span>
                    </div>
                    <div class="stat">
                      <span class="stat-label">CPU</span>
                      <span class="stat-value">{pidInfo.cpu}%</span>
                    </div>
                    <div class="stat">
                      <span class="stat-label">Mem</span>
                      <span class="stat-value">{pidInfo.mem}%</span>
                    </div>
                  </div>
                  <div class="actions">
                    <button
                      class="kill-btn"
                      onclick={() => killProcess(pidInfo.pid)}
                      title="Kill PID {pidInfo.pid}"
                    >
                      Kill
                    </button>
                  </div>
                </div>
              {/each}
            </div>
          </div>
        {/each}
      </div>
      <p class="info">
        {autoRefresh ? "Auto-refreshing every 5 seconds" : "Auto-refresh disabled"}
      </p>
    {/if}
  </div>
</main>

<style>
  .drag-region {
    position: fixed;
    top: 0;
    left: 0;
    right: 0;
    height: 40px;
    z-index: 1000;
  }

  .header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 2rem;
    padding-top: 2rem;
  }

  .controls {
    display: flex;
    gap: 0.5rem;
  }

  .refresh-btn,
  .toggle-btn {
    background: linear-gradient(135deg, #3b82f6 0%, #2563eb 100%);
    color: white;
    border: 1px solid rgba(59, 130, 246, 0.5);
    padding: 0.5rem 1rem;
    border-radius: 6px;
    cursor: pointer;
    font-weight: 600;
    font-size: 0.875rem;
    transition: all 0.2s ease;
    box-shadow: 0 2px 8px rgba(59, 130, 246, 0.3);
  }

  .refresh-btn:hover {
    background: linear-gradient(135deg, #2563eb 0%, #1d4ed8 100%);
    transform: translateY(-1px);
    box-shadow: 0 4px 12px rgba(59, 130, 246, 0.4);
  }

  .toggle-btn {
    background: linear-gradient(135deg, #06b6d4 0%, #0891b2 100%);
    border: 1px solid rgba(6, 182, 212, 0.5);
    box-shadow: 0 2px 8px rgba(6, 182, 212, 0.3);
  }

  .toggle-btn:hover {
    background: linear-gradient(135deg, #0891b2 0%, #0e7490 100%);
    transform: translateY(-1px);
    box-shadow: 0 4px 12px rgba(6, 182, 212, 0.4);
  }

  :global(body) {
    margin: 0;
    padding: 0;
    font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, Oxygen,
      Ubuntu, Cantarell, sans-serif;
    background: linear-gradient(135deg, #050811 0%, #0a1128 50%, #080d1a 100%);
    background-attachment: fixed;
    color: #e0e0e0;
    min-height: 100vh;
  }

  :global(::-webkit-scrollbar) {
    display: none;
  }

  .container {
    max-width: 1400px;
    margin: 0 auto;
    padding: 2rem;
  }

  h1 {
    background: linear-gradient(135deg, #3b82f6 0%, #06b6d4 100%);
    -webkit-background-clip: text;
    -webkit-text-fill-color: transparent;
    background-clip: text;
    margin: 0;
    font-size: 2rem;
    font-weight: 700;
  }

  .error {
    background-color: #ff4444;
    color: white;
    padding: 1rem;
    border-radius: 8px;
    margin-bottom: 1rem;
  }

  .info {
    text-align: center;
    color: #888;
    font-size: 0.9rem;
    margin-top: 1rem;
  }

  .cards {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .card {
    background: rgba(20, 28, 45, 0.88);
    backdrop-filter: blur(10px);
    border: 2px solid rgba(59, 130, 246, 0.5);
    border-radius: 12px;
    overflow: hidden;
    box-shadow:
      0 8px 32px rgba(0, 0, 0, 0.6),
      0 0 20px rgba(59, 130, 246, 0.3),
      0 0 40px rgba(59, 130, 246, 0.15),
      inset 0 0 60px rgba(59, 130, 246, 0.05),
      inset 0 1px 0 rgba(255, 255, 255, 0.04);
    transition: all 0.3s cubic-bezier(0.4, 0, 0.2, 1);
  }

  .card:hover {
    transform: translateY(-4px);
    box-shadow:
      0 12px 48px rgba(0, 0, 0, 0.7),
      0 0 30px rgba(59, 130, 246, 0.5),
      0 0 60px rgba(59, 130, 246, 0.25),
      inset 0 0 80px rgba(59, 130, 246, 0.08),
      inset 0 1px 0 rgba(255, 255, 255, 0.08);
    border-color: rgba(59, 130, 246, 0.8);
  }

  .card-header {
    background: linear-gradient(135deg, rgba(59, 130, 246, 0.12) 0%, rgba(6, 182, 212, 0.12) 100%);
    padding: 1rem 1.5rem;
    border-bottom: 1px solid rgba(59, 130, 246, 0.3);
  }

  .process-info {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .process-name {
    font-family: "Monaco", "Menlo", monospace;
    font-size: 1.1rem;
    font-weight: 700;
    color: #7dd3fc;
    text-shadow: 0 0 8px rgba(125, 211, 252, 0.25);
  }

  .command-wrapper {
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .command-path {
    font-family: "Monaco", "Menlo", monospace;
    font-size: 0.8rem;
    color: #a5b4fc;
    opacity: 0.7;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    transition: all 0.3s ease;
  }

  .command-wrapper:hover .command-path {
    white-space: normal;
    word-break: break-word;
    overflow-wrap: break-word;
  }

  .card-body {
    padding: 0.5rem;
  }

  .pid-row {
    display: flex;
    align-items: center;
    gap: 1.5rem;
    padding: 1rem 1rem;
    border-bottom: 1px solid rgba(59, 130, 246, 0.1);
    transition: all 0.2s ease;
  }

  .pid-row:last-child {
    border-bottom: none;
  }

  .pid-row:hover {
    background: rgba(59, 130, 246, 0.08);
  }

  .pid-info {
    min-width: 80px;
  }

  .pid-label, .port-label {
    font-size: 0.7rem;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: #888;
    margin-bottom: 0.25rem;
  }

  .pid-value {
    font-family: "Monaco", "Menlo", monospace;
    font-size: 1.2rem;
    font-weight: 700;
    color: #fbbf24;
    text-shadow: 0 0 6px rgba(251, 191, 36, 0.2);
  }

  .ports-info {
    flex: 1;
  }

  .port-values {
    display: flex;
    flex-wrap: wrap;
    gap: 0.5rem;
  }

  .port-badge {
    display: inline-flex;
    align-items: center;
    gap: 0.375rem;
    background: linear-gradient(135deg, rgba(34, 197, 94, 0.18) 0%, rgba(5, 150, 105, 0.18) 100%);
    border: 1px solid rgba(34, 197, 94, 0.3);
    color: #6ee7b7;
    padding: 0.375rem 0.75rem;
    border-radius: 6px;
    font-family: "Monaco", "Menlo", monospace;
    font-size: 0.9rem;
    font-weight: 600;
    text-decoration: none;
    transition: all 0.2s ease;
    text-shadow: 0 0 6px rgba(110, 231, 183, 0.25);
  }

  .port-badge.clickable {
    cursor: pointer;
  }

  .port-badge.clickable:hover {
    background: linear-gradient(135deg, rgba(34, 197, 94, 0.28) 0%, rgba(5, 150, 105, 0.28) 100%);
    border-color: rgba(34, 197, 94, 0.5);
    transform: translateY(-1px);
    box-shadow: 0 4px 12px rgba(34, 197, 94, 0.25);
  }

  .stats-info {
    display: flex;
    gap: 2rem;
    margin-left: auto;
  }

  .stat {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .stat-label {
    font-size: 0.7rem;
    text-transform: uppercase;
    letter-spacing: 0.5px;
    color: #888;
  }

  .stat-value {
    font-family: "Monaco", "Menlo", monospace;
    font-size: 0.9rem;
    color: #60a5fa;
    font-weight: 700;
    text-shadow: 0 0 6px rgba(96, 165, 250, 0.3);
  }

  .actions {
    margin-left: 1rem;
  }

  .kill-btn {
    background: linear-gradient(135deg, #ef4444 0%, #dc2626 100%);
    color: white;
    border: 1px solid rgba(239, 68, 68, 0.5);
    padding: 0.5rem 1rem;
    border-radius: 6px;
    cursor: pointer;
    font-weight: 600;
    font-size: 0.875rem;
    transition: all 0.2s ease;
    box-shadow: 0 2px 8px rgba(239, 68, 68, 0.3);
  }

  .kill-btn:hover {
    background: linear-gradient(135deg, #dc2626 0%, #b91c1c 100%);
    transform: translateY(-1px);
    box-shadow: 0 4px 12px rgba(239, 68, 68, 0.4);
  }

  .kill-btn:active {
    transform: translateY(0);
    box-shadow: 0 2px 6px rgba(239, 68, 68, 0.3);
  }

  p {
    text-align: center;
    color: #888;
  }
</style>
