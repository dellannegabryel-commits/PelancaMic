import { useState, useEffect, useCallback } from 'react'
import { invoke } from '@tauri-apps/api/core'
import { listen } from '@tauri-apps/api/event'

interface ServerStatus {
  running: boolean
  clients: number
  audioPort: number
  controlPort: number
  discoveryPort: number
  bufferLen: number
  targetFrames: number
  bufferHealth: number
  packetsReceived: number
  packetsLost: number
  jitterMs: number
}


function App() {
  const [status, setStatus] = useState<ServerStatus>({
    running: false,
    clients: 0,
    audioPort: 41235,
    controlPort: 41236,
    discoveryPort: 41234,
    bufferLen: 0,
    targetFrames: 5,
    bufferHealth: 0,
    packetsReceived: 0,
    packetsLost: 0,
    jitterMs: 0,
  })
  const [logs, setLogs] = useState<string[]>([])
  const [selectedDevice, setSelectedDevice] = useState<string>('')
  const [devices, setDevices] = useState<string[]>([])
  const [loading, setLoading] = useState(false)

  const addLog = useCallback((message: string, type: 'info' | 'success' | 'error' | 'warning' = 'info') => {
    const timestamp = new Date().toLocaleTimeString()
    const prefix = { info: '[INFO]', success: '[OK]', error: '[ERR]', warning: '[WARN]' }[type]
    setLogs(prev => [...prev.slice(-99), `${timestamp} ${prefix} ${message}`])
  }, [])

  const refreshStatus = useCallback(async () => {
    try {
      const serverStatus = await invoke<ServerStatus>('get_status')
      setStatus(serverStatus)
    } catch (e) {
      // Ignore errors when server not running
    }
  }, [])

  const loadDevices = useCallback(async () => {
    try {
      const deviceList = await invoke<string[]>('list_audio_devices')
      setDevices(deviceList)
      setSelectedDevice(current => {
        if (current) return current
        if (deviceList.length === 0) return current
        const vbCable = deviceList.find(d => d.toLowerCase().includes('cable') || d.toLowerCase().includes('vb-audio'))
        return vbCable || deviceList[0]
      })
    } catch (e) {
      addLog(`Failed to load audio devices: ${e}`, 'error')
    }
  }, [addLog])

  const startServer = async () => {
    setLoading(true)
    try {
      await invoke('start_server', { deviceName: selectedDevice || null })
      addLog('Server started on ports 41234 (discovery), 41235 (audio), 41236 (control)', 'success')
      addLog('Waiting for Android client to connect...', 'info')
      setStatus(prev => ({ ...prev, running: true }))
      try {
        const s = await invoke<ServerStatus>('get_status')
        setStatus(s)
      } catch {}
    } catch (e) {
      addLog(`Failed to start server: ${e}`, 'error')
    } finally {
      setLoading(false)
    }
  }

  const stopServer = async () => {
    setLoading(true)
    try {
      await invoke('stop_server')
      addLog('Server stopped', 'warning')
      setStatus(prev => ({ ...prev, running: false, clients: 0 }))
    } catch (e) {
      addLog(`Failed to stop server: ${e}`, 'error')
    } finally {
      setLoading(false)
    }
  }

  // Poll status every second when running
  useEffect(() => {
    if (!status.running) return
    const interval = setInterval(refreshStatus, 1000)
    return () => clearInterval(interval)
  }, [status.running, refreshStatus])

  // Load devices on mount
  useEffect(() => {
    loadDevices()
  }, [loadDevices])

  // Listen for log events from backend
  useEffect(() => {
    const unlisten = listen('log', (event) => {
      const { message, type } = event.payload as { message: string; type: string }
      addLog(message, type as 'info' | 'success' | 'error' | 'warning')
    })
    return () => { unlisten.then(f => f()) }
  }, [addLog])

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
              <div style={{ fontFamily: 'monospace', fontSize: '18px', fontWeight: 600 }}>{status.packetsReceived.toLocaleString()}</div>
            </div>
            <div>
              <label style={{ display: 'block', fontSize: '12px', color: 'var(--fg-secondary)', marginBottom: '4px' }}>Packets Lost</label>
              <div style={{ fontFamily: 'monospace', fontSize: '18px', fontWeight: 600, color: status.packetsLost > 0 ? 'var(--error)' : 'var(--success)' }}>{status.packetsLost}</div>
            </div>
            <div>
              <label style={{ display: 'block', fontSize: '12px', color: 'var(--fg-secondary)', marginBottom: '4px' }}>Jitter</label>
              <div style={{ fontFamily: 'monospace', fontSize: '18px', fontWeight: 600 }}>{status.jitterMs.toFixed(1)} ms</div>
            </div>
            <div>
              <label style={{ display: 'block', fontSize: '12px', color: 'var(--fg-secondary)', marginBottom: '4px' }}>Buffer Health</label>
              <div style={{ fontFamily: 'monospace', fontSize: '18px', fontWeight: 600 }}>{status.bufferHealth.toFixed(0)}%</div>
            </div>
          </div>
          <div style={{ marginTop: '16px', paddingTop: '16px', borderTop: '1px solid var(--border)' }}>
            <label style={{ display: 'block', fontSize: '12px', color: 'var(--fg-secondary)', marginBottom: '4px' }}>
              Buffer: {status.bufferLen} / {status.targetFrames} frames
            </label>
            <div style={{
              height: '8px',
              backgroundColor: 'var(--bg-tertiary)',
              borderRadius: '4px',
              overflow: 'hidden',
            }}>
              <div style={{
                width: `${Math.min(100, status.bufferHealth)}%`,
                height: '100%',
                backgroundColor: status.bufferHealth >= 80 ? 'var(--success)' : status.bufferHealth >= 50 ? 'var(--warning)' : 'var(--error)',
                transition: 'width 0.3s ease',
              }} />
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
              disabled={status.running}
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
              {devices.map(d => (
                <option key={d} value={d}>{d}</option>
              ))}
            </select>
          </div>
          <div style={{ display: 'flex', gap: '12px', flexWrap: 'wrap' }}>
            <button
              onClick={startServer}
              disabled={status.running || loading || !selectedDevice}
              style={{
                padding: '12px 24px',
                borderRadius: '8px',
                border: 'none',
                fontSize: '14px',
                fontWeight: 600,
                cursor: status.running || loading || !selectedDevice ? 'not-allowed' : 'pointer',
                backgroundColor: status.running || loading || !selectedDevice ? 'var(--bg-tertiary)' : 'var(--accent)',
                color: status.running || loading || !selectedDevice ? 'var(--fg-secondary)' : 'var(--bg-primary)',
                opacity: status.running || loading || !selectedDevice ? 0.6 : 1,
              }}
            >
              {loading ? 'Starting...' : status.running ? 'Server Running' : 'Start Server'}
            </button>
            <button
              onClick={stopServer}
              disabled={!status.running || loading}
              style={{
                padding: '12px 24px',
                borderRadius: '8px',
                border: '1px solid var(--error)',
                fontSize: '14px',
                fontWeight: 600,
                cursor: !status.running || loading ? 'not-allowed' : 'pointer',
                backgroundColor: !status.running || loading ? 'var(--bg-tertiary)' : 'transparent',
                color: !status.running || loading ? 'var(--fg-secondary)' : 'var(--error)',
                opacity: !status.running || loading ? 0.6 : 1,
              }}
            >
              {loading ? 'Stopping...' : 'Stop Server'}
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