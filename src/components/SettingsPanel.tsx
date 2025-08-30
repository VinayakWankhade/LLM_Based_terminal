import React, { useEffect, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface Keybindings {
  open_ai_panel: string;
  open_workflows: string;
  split_vertical: string;
  split_horizontal: string;
  close_pane: string;
}

interface Settings {
  theme: string;
  font_size: number;
  telemetry_enabled: boolean;
  analytics_endpoint?: string | null;
  keybindings: Keybindings;
}

interface Props {
  visible: boolean;
  onClose: () => void;
  onSaved?: (settings: Settings) => void;
}

const SettingsPanel: React.FC<Props> = ({ visible, onClose, onSaved }) => {
  const [settings, setSettings] = useState<Settings | null>(null);
  const [saving, setSaving] = useState(false);

  useEffect(() => {
    if (!visible) return;
    (async () => {
      try {
        const s = await invoke<Settings>('get_settings');
        setSettings(s);
      } catch (e) {
        console.error('Failed to load settings', e);
      }
    })();
  }, [visible]);

  const save = async () => {
    if (!settings) return;
    setSaving(true);
    try {
      await invoke('save_user_settings', { settings });
      onSaved?.(settings);
      onClose();
    } catch (e) {
      console.error('Failed to save settings', e);
    } finally {
      setSaving(false);
    }
  };

  if (!visible) return null;

  return (
    <div className="search-results-overlay">
      <div className="search-results-modal" style={{ maxWidth: 800 }}>
        <div className="results-header">
          <div className="results-title">Settings</div>
          <div className="results-actions">
            <button className="terminal-control-btn" onClick={save} disabled={!settings || saving}>Save</button>
            <button className="close-button" onClick={onClose}>×</button>
          </div>
        </div>
        <div className="editor-body">
          {!settings ? (
            <div style={{ color: 'var(--muted)' }}>Loading...</div>
          ) : (
            <>
              <div style={{ display: 'flex', gap: 12 }}>
                <div style={{ flex: 1 }}>
                  <label style={{ display: 'block', fontSize: 12, color: 'var(--muted)' }}>Theme</label>
                  <select
                    className="search-input"
                    value={settings.theme}
                    onChange={(e) => setSettings({ ...settings, theme: e.target.value })}
                  >
                    <option value="dark">Dark</option>
                    <option value="light">Light</option>
                  </select>
                </div>
                <div style={{ width: 160 }}>
                  <label style={{ display: 'block', fontSize: 12, color: 'var(--muted)' }}>Font size</label>
                  <input
                    className="search-input"
                    type="number"
                    min={10}
                    max={24}
                    value={settings.font_size}
                    onChange={(e) => setSettings({ ...settings, font_size: Math.max(10, Math.min(24, Number(e.target.value)||14)) })}
                  />
                </div>
              </div>

              <div>
                <label className="search-toggle">
                  <input
                    type="checkbox"
                    checked={settings.telemetry_enabled}
                    onChange={(e) => setSettings({ ...settings, telemetry_enabled: e.target.checked })}
                  />
                  Enable telemetry (anonymous events)
                </label>
              </div>

              <div style={{ marginTop: 8 }}>
                <div style={{ fontWeight: 600, marginBottom: 6 }}>Keybindings</div>
                <div style={{ display: 'grid', gridTemplateColumns: '1fr 1fr', gap: 8 }}>
                  <div>
                    <label style={{ display: 'block', fontSize: 12, color: 'var(--muted)' }}>Toggle AI Panel</label>
                    <input className="search-input" value={settings.keybindings.open_ai_panel}
                      onChange={(e) => setSettings({ ...settings, keybindings: { ...settings.keybindings, open_ai_panel: e.target.value } })} />
                  </div>
                  <div>
                    <label style={{ display: 'block', fontSize: 12, color: 'var(--muted)' }}>Toggle Workflows</label>
                    <input className="search-input" value={settings.keybindings.open_workflows}
                      onChange={(e) => setSettings({ ...settings, keybindings: { ...settings.keybindings, open_workflows: e.target.value } })} />
                  </div>
                  <div>
                    <label style={{ display: 'block', fontSize: 12, color: 'var(--muted)' }}>Split Vertical</label>
                    <input className="search-input" value={settings.keybindings.split_vertical}
                      onChange={(e) => setSettings({ ...settings, keybindings: { ...settings.keybindings, split_vertical: e.target.value } })} />
                  </div>
                  <div>
                    <label style={{ display: 'block', fontSize: 12, color: 'var(--muted)' }}>Split Horizontal</label>
                    <input className="search-input" value={settings.keybindings.split_horizontal}
                      onChange={(e) => setSettings({ ...settings, keybindings: { ...settings.keybindings, split_horizontal: e.target.value } })} />
                  </div>
                  <div>
                    <label style={{ display: 'block', fontSize: 12, color: 'var(--muted)' }}>Close Pane</label>
                    <input className="search-input" value={settings.keybindings.close_pane}
                      onChange={(e) => setSettings({ ...settings, keybindings: { ...settings.keybindings, close_pane: e.target.value } })} />
                  </div>
                </div>
              </div>
            </>
          )}
        </div>
      </div>
    </div>
  );
};

export default SettingsPanel;
