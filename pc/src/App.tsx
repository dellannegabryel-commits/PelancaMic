import { useState, useEffect } from 'react'

interface ServerStatus {
  running: boolean
  clients: number
  audioPort: number
  controlPort: number
  discoveryPort: number
}

interface AudioStats {
  packetsReceived: number
  packetsLost: number
  jitterMs: number
  bufferHealth: number
}

function App() {
  const [status, setStatus] = useState<ServerStatus>({
    running: false,
    clients: 0,
    audioPort: 41235,
    controlPort: 41236,
    discoveryPort: 41234,
  })
  const [audioStats, setAudioStats] = useState<AudioStats>({
    packetsReceived: 0,
    packetsLost: 0,
    jitterMs: 0,
    bufferHealth: 0,
  })
  const [logs, setLogs] = useState<string[]>([])
  const [selectedDevice, setSelectedDevice] = useState<string>('')

  const addLog = (message: string, type: 'info' | 'success' | 'error' | 'warning' = 'info') => {
    const timestamp = new Date().toLocaleTimeString()
    const prefix = { info: '[INFO]', success: '[OK]', error: '[ERR]', warning: '[WARN]' }[type]
    setLogs(prev => [...prev.slice(-99), `${timestamp} ${prefix} ${message}`])
  }

  const startServer = async () => {
    try {
      // TODO: Invoke Tauri command to start Rust servers
      setStatus(prev => ({ ...prev, running: true }))
      addLog('Server started on ports 41234 (discovery), 41235 (audio), 41236 (control)', 'success')
      addLog('Waiting for Android client to connect...', 'info')
    } catch (e) {
      addLog(`Failed to start server: ${e}`, 'error')
    }
  }

  const stopServer = async () => {
    try {
      // TODO: Invoke Tauri command to stop Rust servers
      setStatus(prev => ({ ...prev, running: false, clients: 0 }))
      addLog('Server stopped', 'warning')
    } catch (e) {
      addLog(`Failed to stop server: ${e}`, 'error')
    }
  }

  // Simulate stats updates
  useEffect(() => {
    if (!status.running) return
    const interval = setInterval(() => {
      setAudioStats(prev => ({
        packetsReceived: prev.packetsReceived + Math.floor(Math.random() * 10) + 48,
        packetsLost: prev.packetsLost + (Math.random() > 0.95 ? 1 : 0),
        jitterMs: Math.max(1, Math.min(15, prev.jitterMs + (Math.random() - 0.5) * 2)),
        bufferHealth: Math.max(30, Math.min(100, prev.bufferHealth + (Math.random() - 0.5) * 10)),
      }))
    }, 1000)
    return () => clearInterval(interval)
  }, [status.running])

  return (
    <div style={{ display: 'flex', flexDirection: 'column', minHeight: '100vh' }}>
      <header style={{
        padding: '16px 24px',
        backgroundColor: 'var(--bg-secondary)',
        borderBottom: '1px solid var(--border)',
        display: 'flex',
        justifyContent: 'space-between',
        alignItems: 'center',
      }}>
        <h1 style={{ fontSize: '20px', fontWeight: 600, color: 'var(--accent)' }}>PelancaMic Server</h1>
        <div style={{ display: 'flex', gap: '12px', alignItems: 'center' }}>
          <span style={{
            padding: '4px 12px',
            borderRadius: '12px',
            fontSize: '12px',
            fontWeight: 500,
            backgroundColor: status.running ? 'rgba(0, 212, 170, 0.15)' : 'rgba(255, 68, 68, 0.15)',
            color: status.running ? 'var(--success)' : 'var(--error)',
          }}>
            {status.running ? 'RUNNING' : 'STOPPED'}
          </span>
        </div>
      </header>

      <main style={{ flex: 1, padding: '24px', display: 'grid', gridTemplateColumns: 'repeat(auto-fit, minmax(300px, 1fr))', gap: '24px' }}>
        <section style={{
          backgroundColor: 'var(--bg-secondary)',
          border: '1px solid var(--border)',
          borderRadius: '12px',
          padding: '20px',
        }}>
          <h2 style={{ fontSize: '16px', fontWeight: 600, marginBottom: '16px', color: 'var(--fg-secondary)' }}>Network Configuration</h2>
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(2, 1fr)', gap: '12px' }}>
            <div>
              <label style={{ display: 'block', fontSize: '12px', color: 'var(--fg-secondary)', marginBottom: '4px' }}>Discovery (UDP)</label>
              <div style={{ fontFamily: 'monospace', fontSize: '16px', fontWeight: 500 }}>{status.discoveryPort}</div>
            </div>
            <div>
              <label style={{ display: 'block', fontSize: '12px', color: 'var(--fg-secondary)', marginBottom: '4px' }}>Audio (UDP)</label>
              <div style={{ fontFamily: 'monospace', fontSize: '16px', fontWeight: 500 }}>{status.audioPort}</div>
            </div>
            <div>
              <label style={{ display: 'block', fontSize: '12px', color: 'var(--fg-secondary)', marginBottom: '4px' }}>Control (WS)</label>
              <div style={{ fontFamily: 'monospace', fontSize: '16px', fontWeight: 500 }}>{status.controlPort}</div>
            </div>
            <div>
              <label style={{ display: 'block', fontSize: '12px', color: 'var(--fg-secondary)', marginBottom: '4px' }}>Connected Clients</label>
              <div style={{ fontFamily: 'monospace', fontSize: '16px', fontWeight: 500, color: 'var(--accent)' }}>{status.clients}</div>
            </div>
          </div>
        </section>

        <section style={{
          backgroundColor: 'var(--bg-secondary)',
          border: '1px solid var(--border)',
          borderRadius: '12px',
          padding: '20px',
        }}>
          <h2 style={{ fontSize: '16px', fontWeight: 600, marginBottom: '16px', color: 'var(--fg-secondary)' }}>Audio Statistics</h2>
          <div style={{ display: 'grid', gridTemplateColumns: 'repeat(2, 1fr)', gap: '12px' }}>
            <div>
              <label style={{ display: 'block', fontSize: '12px', color: 'var(--fg-secondary)', marginBottom: '4px' }}>Packets Received</label>
              <div style={{ fontFamily: 'monospace', fontSize: '18px', fontWeight: 600 }}>{audioStats.packetsReceived.toLocaleString()}</div>
            </div>
            <div>
              <label style={{ display: 'block', fontSize: '12px', color: 'var(--fg-secondary)', marginBottom: '4px' }}>Packets Lost</label>
              <div style={{ fontFamily: 'monospace', fontSize: '18px', fontWeight: 600, color: audioStats.packetsLost > 0 ? 'var(--error)' : 'var(--success)' }}>{audioStats.packetsLost}</div>
            </div>
            <div>
              <label style={{ display: 'block', fontSize: '12px', color: 'var(--fg-secondary)', marginBottom: '4px' }}>Jitter</label>
              <div style={{ fontFamily: 'monospace', fontSize: '18px', fontWeight: 600 }}>{audioStats.jitterMs.toFixed(1)} ms</div>
            </div>
            <div>
              <label style={{ display: 'block', fontSize: '12px', color: 'var(--fg-secondary)', marginBottom: '4px' }}>Buffer Health</label>
              <div style={{ fontFamily: 'monospace', fontSize: '18px', fontWeight: 600 }}>{audioStats.bufferHealth.toFixed(0)}%</div>
            </div>
          </div>
        </section>

        <section style={{
          backgroundColor: 'var(--bg-secondary)',
          border: '1px solid var(--border)',
          borderRadius: '12px',
          padding: '20px',
          gridColumn: '1 / -1',
        }}>
          <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '16px' }}>
            <h2 style={{ fontSize: '16px', fontWeight: 600, color: 'var(--fg-secondary)' }}>Audio Output Device</h2>
            <select
              value={selectedDevice}
              onChange={e => setSelectedDevice(e.target.value)}
              style={{
                padding: '8px 12px',
                borderRadius: '6px',
                backgroundColor: 'var(--bg-tertiary)',
                border: '1px solid var(--border)',
                color: 'var(--fg-primary)',
                fontSize: '14px',
              }}
            >
              <option value="">Select virtual cable...</option>
              <option value="CABLE Input (VB-Audio Virtual Cable)">CABLE Input (VB-Audio Virtual Cable)</option>
              <option value="VB-Audio Point">VB-Audio Point</option>
              <option value="PipeWire Loopback">PipeWire Loopback (Linux)</option>
            </select>
          </div>
          <div style={{ display: 'flex', gap: '12px', flexWrap: 'wrap' }}>
            <button
              onClick={startServer}
              disabled={status.running}
              style={{
                padding: '12px 24px',
                borderRadius: '8px',
                border: 'none',
                fontSize: '14px',
                fontWeight: 600,
                cursor: status.running ? 'not-allowed' : 'pointer',
                backgroundColor: status.running ? 'var(--bg-tertiary)' : 'var(--accent)',
                color: status.running ? 'var(--fg-secondary)' : 'var(--bg-primary)',
                opacity: status.running ? 0.6 : 1,
              }}
            >
              {status.running ? 'Server Running' : 'Start Server'}
            </button>
            <button
              onClick={stopServer}
              disabled={!status.running}
              style={{
                padding: '12px 24px',
                borderRadius: '8px',
                border: '1px solid var(--error)',
                fontSize: '14px',
                fontWeight: 600,
                cursor: !status.running ? 'not-allowed' : 'pointer',
                backgroundColor: !status.running ? 'var(--bg-tertiary)' : 'transparent',
                color: !status.running ? 'var(--fg-secondary)' : 'var(--error)',
                opacity: !status.running ? 0.6 : 1,
              }}
            >
              Stop Server
            </button>
          </div>
        </section>

        <section style={{
          backgroundColor: 'var(--bg-secondary)',
          border: '1px solid var(--border)',
          borderRadius: '12px',
          padding: '20px',
          gridColumn: '1 / -1',
        }}>
          <h2 style={{ fontSize: '16px', fontWeight: 600, marginBottom: '12px', color: 'var(--fg-secondary)' }}>Logs</h2>
          <div style={{
            backgroundColor: 'var(--bg-primary)',
            border: '1px solid var(--border)',
            borderRadius: '8px',
            padding: '12px',
            maxHeight: '200px',
            overflowY: 'auto',
            fontFamily: 'monospace',
            fontSize: '12px',
            lineHeight: 1.6,
          }}>
            {logs.length === 0 && <span style={{ color: 'var(--fg-secondary)' }}>No logs yet. Start the server to see activity.</span>}
            {logs.map((log, i) => (
              <div key={i} style={{ color: log.includes('[OK]') ? 'var(--success)' : log.includes('[ERR]') ? 'var(--error)' : log.includes('[WARN]') ? 'var(--warning)' : 'var(--fg-primary)' }}>
                {log}
              </div>
            ))}
          </div>
        </section>
      </main>
    </div>
  )
}

export default App