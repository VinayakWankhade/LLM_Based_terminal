import React, { useState, useEffect } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface PerformanceMetrics {
  terminal_id: string;
  timestamp: number;
  cpu_usage: number;
  memory_usage: number;
  output_rate: number;
  input_rate: number;
  render_time_ms: number;
  latency_ms: number;
  scrollback_size: number;
  active_processes: number;
  bandwidth_in: number;
  bandwidth_out: number;
}

interface SecurityAlert {
  id: string;
  terminal_id: string;
  alert_type: string;
  message: string;
  timestamp: number;
  risk_level: string;
  command?: string;
  remediation?: string;
}

interface SessionInfo {
  id: string;
  name: string;
  created_at: string;
  last_accessed: string;
  is_detached: boolean;
  window_title?: string;
  tabs: TabInfo[];
  active_tab_id?: string;
}

interface TabInfo {
  id: string;
  title: string;
  working_dir: string;
  shell: string;
  panes: PaneInfo[];
  active_pane_id?: string;
}

interface PaneInfo {
  id: string;
  terminal_id: string;
  working_dir: string;
  command_history: string[];
  scrollback_lines: number;
}

interface AdvancedFeaturesProps {
  terminalId: string;
  visible: boolean;
  onClose: () => void;
}

const AdvancedFeatures: React.FC<AdvancedFeaturesProps> = ({ terminalId, visible, onClose }) => {
  const [activeTab, setActiveTab] = useState<'performance' | 'security' | 'sessions' | 'accessibility'>('performance');
  const [performanceMetrics, setPerformanceMetrics] = useState<PerformanceMetrics[]>([]);
  const [securityAlerts, setSecurityAlerts] = useState<SecurityAlert[]>([]);
  const [sessions, setSessions] = useState<SessionInfo[]>([]);
  const [isMonitoring, setIsMonitoring] = useState(false);

  useEffect(() => {
    if (visible) {
      loadPerformanceMetrics();
      loadSecurityAlerts();
      loadSessions();
    }
  }, [visible, terminalId]);

  const loadPerformanceMetrics = async () => {
    try {
      const metrics = await invoke<PerformanceMetrics[]>('get_performance_metrics', {
        terminalId,
        durationSeconds: 300 // Last 5 minutes
      });
      setPerformanceMetrics(metrics);
    } catch (error) {
      console.error('Failed to load performance metrics:', error);
    }
  };

  const loadSecurityAlerts = async () => {
    try {
      const alerts = await invoke<SecurityAlert[]>('get_security_alerts', { limit: 50 });
      setSecurityAlerts(alerts);
    } catch (error) {
      console.error('Failed to load security alerts:', error);
    }
  };

  const loadSessions = async () => {
    try {
      const sessionList = await invoke<SessionInfo[]>('list_sessions');
      setSessions(sessionList);
    } catch (error) {
      console.error('Failed to load sessions:', error);
    }
  };

  const togglePerformanceMonitoring = async () => {
    try {
      await invoke('toggle_performance_monitoring', { enabled: !isMonitoring });
      setIsMonitoring(!isMonitoring);
    } catch (error) {
      console.error('Failed to toggle performance monitoring:', error);
    }
  };

  const createNewSession = async () => {
    try {
      const sessionId = await invoke<string>('create_session', {
        name: `Session ${sessions.length + 1}`,
        shell: null,
        workingDir: null
      });
      await loadSessions();
      console.log('Created new session:', sessionId);
    } catch (error) {
      console.error('Failed to create session:', error);
    }
  };

  const attachToSession = async (sessionId: string) => {
    try {
      await invoke('attach_session', { sessionId });
      console.log('Attached to session:', sessionId);
    } catch (error) {
      console.error('Failed to attach to session:', error);
    }
  };

  const detachFromSession = async (sessionId: string) => {
    try {
      await invoke('detach_session', { sessionId });
      await loadSessions();
      console.log('Detached from session:', sessionId);
    } catch (error) {
      console.error('Failed to detach from session:', error);
    }
  };

  if (!visible) return null;

  const formatBytes = (bytes: number) => {
    const sizes = ['B', 'KB', 'MB', 'GB'];
    if (bytes === 0) return '0 B';
    const i = Math.floor(Math.log(bytes) / Math.log(1024));
    return `${(bytes / Math.pow(1024, i)).toFixed(1)} ${sizes[i]}`;
  };

  const formatTimestamp = (timestamp: number) => {
    return new Date(timestamp).toLocaleString();
  };

  const getRiskLevelColor = (level: string) => {
    switch (level.toLowerCase()) {
      case 'low': return '#10b981';
      case 'medium': return '#f59e0b';
      case 'high': return '#ef4444';
      case 'critical': return '#dc2626';
      default: return '#6b7280';
    }
  };

  return (
    <div className="search-results-overlay">
      <div className="search-results-modal" style={{ maxWidth: 1200, height: '90%' }}>
        <div className="results-header">
          <div className="results-title">Advanced Terminal Features</div>
          <div className="results-actions">
            <button className="close-button" onClick={onClose}>×</button>
          </div>
        </div>

        <div style={{ display: 'flex', height: '100%' }}>
          {/* Sidebar Navigation */}
          <div style={{ width: 200, borderRight: '1px solid var(--border)', padding: '12px' }}>
            <nav style={{ display: 'flex', flexDirection: 'column', gap: '8px' }}>
              <button
                className={`terminal-control-btn ${activeTab === 'performance' ? 'active' : ''}`}
                onClick={() => setActiveTab('performance')}
                style={{ textAlign: 'left', justifyContent: 'flex-start' }}
              >
                📊 Performance
              </button>
              <button
                className={`terminal-control-btn ${activeTab === 'security' ? 'active' : ''}`}
                onClick={() => setActiveTab('security')}
                style={{ textAlign: 'left', justifyContent: 'flex-start' }}
              >
                🔒 Security
              </button>
              <button
                className={`terminal-control-btn ${activeTab === 'sessions' ? 'active' : ''}`}
                onClick={() => setActiveTab('sessions')}
                style={{ textAlign: 'left', justifyContent: 'flex-start' }}
              >
                🖥️ Sessions
              </button>
              <button
                className={`terminal-control-btn ${activeTab === 'accessibility' ? 'active' : ''}`}
                onClick={() => setActiveTab('accessibility')}
                style={{ textAlign: 'left', justifyContent: 'flex-start' }}
              >
                ♿ Accessibility
              </button>
            </nav>
          </div>

          {/* Main Content */}
          <div style={{ flex: 1, padding: '16px', overflow: 'auto' }}>
            {activeTab === 'performance' && (
              <div>
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '16px' }}>
                  <h3>Performance Monitoring</h3>
                  <div style={{ display: 'flex', gap: '8px', alignItems: 'center' }}>
                    <span style={{ fontSize: '12px', color: 'var(--muted)' }}>
                      Monitoring: {isMonitoring ? '✅ Enabled' : '❌ Disabled'}
                    </span>
                    <button className="terminal-control-btn" onClick={togglePerformanceMonitoring}>
                      {isMonitoring ? 'Disable' : 'Enable'} Monitoring
                    </button>
                    <button className="terminal-control-btn" onClick={loadPerformanceMetrics}>
                      🔄 Refresh
                    </button>
                  </div>
                </div>

                {performanceMetrics.length > 0 ? (
                  <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: '16px' }}>
                    <div className="performance-card">
                      <h4>Resource Usage</h4>
                      <div className="metric-grid">
                        <div className="metric">
                          <span className="metric-label">CPU Usage</span>
                          <span className="metric-value">
                            {performanceMetrics[performanceMetrics.length - 1]?.cpu_usage.toFixed(1)}%
                          </span>
                        </div>
                        <div className="metric">
                          <span className="metric-label">Memory Usage</span>
                          <span className="metric-value">
                            {formatBytes(performanceMetrics[performanceMetrics.length - 1]?.memory_usage || 0)}
                          </span>
                        </div>
                        <div className="metric">
                          <span className="metric-label">Latency</span>
                          <span className="metric-value">
                            {performanceMetrics[performanceMetrics.length - 1]?.latency_ms.toFixed(1)}ms
                          </span>
                        </div>
                        <div className="metric">
                          <span className="metric-label">Active Processes</span>
                          <span className="metric-value">
                            {performanceMetrics[performanceMetrics.length - 1]?.active_processes || 0}
                          </span>
                        </div>
                      </div>
                    </div>

                    <div className="performance-card">
                      <h4>I/O Statistics</h4>
                      <div className="metric-grid">
                        <div className="metric">
                          <span className="metric-label">Output Rate</span>
                          <span className="metric-value">
                            {formatBytes(performanceMetrics[performanceMetrics.length - 1]?.output_rate || 0)}/s
                          </span>
                        </div>
                        <div className="metric">
                          <span className="metric-label">Input Rate</span>
                          <span className="metric-value">
                            {formatBytes(performanceMetrics[performanceMetrics.length - 1]?.input_rate || 0)}/s
                          </span>
                        </div>
                        <div className="metric">
                          <span className="metric-label">Scrollback Size</span>
                          <span className="metric-value">
                            {formatBytes(performanceMetrics[performanceMetrics.length - 1]?.scrollback_size || 0)}
                          </span>
                        </div>
                        <div className="metric">
                          <span className="metric-label">Render Time</span>
                          <span className="metric-value">
                            {performanceMetrics[performanceMetrics.length - 1]?.render_time_ms.toFixed(1)}ms
                          </span>
                        </div>
                      </div>
                    </div>
                  </div>
                ) : (
                  <div style={{ textAlign: 'center', padding: '40px', color: 'var(--muted)' }}>
                    No performance data available. Enable monitoring to start collecting metrics.
                  </div>
                )}
              </div>
            )}

            {activeTab === 'security' && (
              <div>
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '16px' }}>
                  <h3>Security & Audit</h3>
                  <button className="terminal-control-btn" onClick={loadSecurityAlerts}>
                    🔄 Refresh Alerts
                  </button>
                </div>

                <div style={{ marginBottom: '24px' }}>
                  <h4>Recent Security Alerts</h4>
                  <div className="security-alerts">
                    {securityAlerts.length > 0 ? (
                      securityAlerts.map(alert => (
                        <div key={alert.id} className="security-alert" style={{ borderLeft: `4px solid ${getRiskLevelColor(alert.risk_level)}` }}>
                          <div className="alert-header">
                            <span className="alert-type">{alert.alert_type}</span>
                            <span className="alert-timestamp">{formatTimestamp(alert.timestamp)}</span>
                          </div>
                          <div className="alert-message">{alert.message}</div>
                          {alert.command && (
                            <div className="alert-command">Command: <code>{alert.command}</code></div>
                          )}
                          {alert.remediation && (
                            <div className="alert-remediation">💡 {alert.remediation}</div>
                          )}
                        </div>
                      ))
                    ) : (
                      <div style={{ textAlign: 'center', padding: '20px', color: 'var(--muted)' }}>
                        No security alerts found.
                      </div>
                    )}
                  </div>
                </div>

                <div>
                  <h4>Security Policy</h4>
                  <div className="security-policy">
                    <div className="policy-item">
                      <label>
                        <input type="checkbox" defaultChecked />
                        Enable audit logging
                      </label>
                    </div>
                    <div className="policy-item">
                      <label>
                        <input type="checkbox" defaultChecked />
                        Mask sensitive data in logs
                      </label>
                    </div>
                    <div className="policy-item">
                      <label>
                        <input type="checkbox" />
                        Require confirmation for dangerous commands
                      </label>
                    </div>
                    <div className="policy-item">
                      <label>
                        <input type="checkbox" />
                        Enable session encryption
                      </label>
                    </div>
                  </div>
                </div>
              </div>
            )}

            {activeTab === 'sessions' && (
              <div>
                <div style={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center', marginBottom: '16px' }}>
                  <h3>Session Management</h3>
                  <div style={{ display: 'flex', gap: '8px' }}>
                    <button className="terminal-control-btn" onClick={createNewSession}>
                      ➕ New Session
                    </button>
                    <button className="terminal-control-btn" onClick={loadSessions}>
                      🔄 Refresh
                    </button>
                  </div>
                </div>

                <div className="sessions-list">
                  {sessions.length > 0 ? (
                    sessions.map(session => (
                      <div key={session.id} className="session-item">
                        <div className="session-header">
                          <div className="session-name">
                            <strong>{session.name}</strong>
                            {session.is_detached && <span className="session-badge">Detached</span>}
                          </div>
                          <div className="session-actions">
                            <button 
                              className="terminal-control-btn"
                              onClick={() => attachToSession(session.id)}
                              disabled={!session.is_detached}
                            >
                              📎 Attach
                            </button>
                            <button 
                              className="terminal-control-btn"
                              onClick={() => detachFromSession(session.id)}
                              disabled={session.is_detached}
                            >
                              📤 Detach
                            </button>
                          </div>
                        </div>
                        <div className="session-details">
                          <div className="session-info">
                            <span>Created: {formatTimestamp(new Date(session.created_at).getTime())}</span>
                            <span>Last accessed: {formatTimestamp(new Date(session.last_accessed).getTime())}</span>
                          </div>
                          <div className="session-tabs">
                            <span>Tabs: {session.tabs.length}</span>
                            <span>Total panes: {session.tabs.reduce((acc, tab) => acc + tab.panes.length, 0)}</span>
                          </div>
                        </div>
                      </div>
                    ))
                  ) : (
                    <div style={{ textAlign: 'center', padding: '40px', color: 'var(--muted)' }}>
                      No sessions available. Create a new session to get started.
                    </div>
                  )}
                </div>
              </div>
            )}

            {activeTab === 'accessibility' && (
              <div>
                <h3>Accessibility Settings</h3>
                <div className="accessibility-settings">
                  <div className="setting-group">
                    <h4>Visual Accessibility</h4>
                    <div className="setting-item">
                      <label>
                        <input type="checkbox" />
                        High contrast mode
                      </label>
                    </div>
                    <div className="setting-item">
                      <label>
                        <input type="checkbox" />
                        Large cursor
                      </label>
                    </div>
                    <div className="setting-item">
                      <label>
                        <input type="checkbox" />
                        Reduce motion
                      </label>
                    </div>
                    <div className="setting-item">
                      <label>
                        Font scale:
                        <select style={{ marginLeft: '8px' }}>
                          <option value="0.8">80%</option>
                          <option value="1.0" selected>100%</option>
                          <option value="1.2">120%</option>
                          <option value="1.5">150%</option>
                          <option value="2.0">200%</option>
                        </select>
                      </label>
                    </div>
                  </div>

                  <div className="setting-group">
                    <h4>Screen Reader Support</h4>
                    <div className="setting-item">
                      <label>
                        <input type="checkbox" />
                        Enable screen reader announcements
                      </label>
                    </div>
                    <div className="setting-item">
                      <label>
                        <input type="checkbox" />
                        Announce command completions
                      </label>
                    </div>
                    <div className="setting-item">
                      <label>
                        <input type="checkbox" />
                        Announce directory changes
                      </label>
                    </div>
                  </div>

                  <div className="setting-group">
                    <h4>Keyboard Navigation</h4>
                    <div className="setting-item">
                      <label>
                        <input type="checkbox" defaultChecked />
                        Enable keyboard shortcuts
                      </label>
                    </div>
                    <div className="setting-item">
                      <label>
                        <input type="checkbox" />
                        Sticky keys support
                      </label>
                    </div>
                    <div className="setting-item">
                      <label>
                        <input type="checkbox" />
                        Show keyboard focus indicators
                      </label>
                    </div>
                  </div>
                </div>
              </div>
            )}
          </div>
        </div>
      </div>

      <style>{`
        .performance-card {
          background: var(--panel-bg);
          border: 1px solid var(--border);
          border-radius: 8px;
          padding: 16px;
        }

        .metric-grid {
          display: grid;
          grid-template-columns: 1fr 1fr;
          gap: 12px;
          margin-top: 12px;
        }

        .metric {
          display: flex;
          justify-content: space-between;
          padding: 8px;
          background: var(--panel-alt);
          border-radius: 4px;
        }

        .metric-label {
          color: var(--muted);
          font-size: 12px;
        }

        .metric-value {
          font-weight: 600;
          font-family: 'Fira Code', monospace;
        }

        .security-alerts {
          max-height: 300px;
          overflow-y: auto;
        }

        .security-alert {
          background: var(--panel-bg);
          border: 1px solid var(--border);
          border-radius: 6px;
          padding: 12px;
          margin-bottom: 12px;
        }

        .alert-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 8px;
        }

        .alert-type {
          font-weight: 600;
          text-transform: capitalize;
        }

        .alert-timestamp {
          font-size: 11px;
          color: var(--muted);
        }

        .alert-message {
          margin-bottom: 8px;
        }

        .alert-command {
          font-size: 12px;
          color: var(--muted);
          margin-bottom: 4px;
        }

        .alert-remediation {
          font-size: 12px;
          color: var(--accent-2);
        }

        .security-policy {
          display: flex;
          flex-direction: column;
          gap: 8px;
        }

        .policy-item label {
          display: flex;
          align-items: center;
          gap: 8px;
          cursor: pointer;
        }

        .sessions-list {
          display: flex;
          flex-direction: column;
          gap: 12px;
        }

        .session-item {
          background: var(--panel-bg);
          border: 1px solid var(--border);
          border-radius: 8px;
          padding: 16px;
        }

        .session-header {
          display: flex;
          justify-content: space-between;
          align-items: center;
          margin-bottom: 12px;
        }

        .session-name {
          display: flex;
          align-items: center;
          gap: 8px;
        }

        .session-badge {
          background: var(--accent);
          color: white;
          padding: 2px 8px;
          border-radius: 12px;
          font-size: 11px;
          font-weight: 500;
        }

        .session-actions {
          display: flex;
          gap: 8px;
        }

        .session-details {
          display: flex;
          justify-content: space-between;
          font-size: 12px;
          color: var(--muted);
        }

        .session-info {
          display: flex;
          flex-direction: column;
          gap: 4px;
        }

        .session-tabs {
          display: flex;
          flex-direction: column;
          gap: 4px;
        }

        .accessibility-settings {
          display: flex;
          flex-direction: column;
          gap: 24px;
        }

        .setting-group {
          background: var(--panel-bg);
          border: 1px solid var(--border);
          border-radius: 8px;
          padding: 16px;
        }

        .setting-group h4 {
          margin-bottom: 12px;
          color: var(--fg);
        }

        .setting-item {
          margin-bottom: 12px;
        }

        .setting-item:last-child {
          margin-bottom: 0;
        }

        .setting-item label {
          display: flex;
          align-items: center;
          gap: 8px;
          cursor: pointer;
        }
      `}</style>
    </div>
  );
};

export default AdvancedFeatures;
