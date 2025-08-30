import React, { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface Command {
  id: string;
  text: string;
  timestamp: number;
  working_dir: string;
  exit_code?: number;
  duration_ms?: number;
  shell_type: string;
}

interface CommandHistoryProps {
  terminalId: string;
  visible: boolean;
  onClose: () => void;
  onCommandSelect: (command: string) => void;
}

const CommandHistory: React.FC<CommandHistoryProps> = ({
  terminalId,
  visible,
  onClose,
  onCommandSelect,
}) => {
  const [history, setHistory] = useState<Command[]>([]);
  const [filteredHistory, setFilteredHistory] = useState<Command[]>([]);
  const [searchQuery, setSearchQuery] = useState('');
  const [selectedIndex, setSelectedIndex] = useState(0);
  const [loading, setLoading] = useState(false);
  const historyRef = useRef<HTMLDivElement>(null);
  const searchRef = useRef<HTMLInputElement>(null);

  // Load command history when component becomes visible
  useEffect(() => {
    if (visible) {
      loadHistory();
      // Focus search input
      setTimeout(() => {
        searchRef.current?.focus();
      }, 100);
    }
  }, [visible, terminalId]);

  // Filter history based on search query
  useEffect(() => {
    if (searchQuery.trim() === '') {
      setFilteredHistory(history);
    } else {
      const filtered = history.filter(cmd =>
        cmd.text.toLowerCase().includes(searchQuery.toLowerCase()) ||
        cmd.working_dir.toLowerCase().includes(searchQuery.toLowerCase())
      );
      setFilteredHistory(filtered);
    }
    setSelectedIndex(0);
  }, [history, searchQuery]);

  const loadHistory = async () => {
    setLoading(true);
    try {
      const commands = await invoke<Command[]>('get_command_history', {
        terminalId,
        limit: 100,
      });
      setHistory(commands);
    } catch (error) {
      console.error('Failed to load command history:', error);
      setHistory([]);
    } finally {
      setLoading(false);
    }
  };


  const handleKeyDown = (event: React.KeyboardEvent) => {
    switch (event.key) {
      case 'ArrowDown':
        event.preventDefault();
        setSelectedIndex(prev => Math.min(prev + 1, filteredHistory.length - 1));
        break;
      case 'ArrowUp':
        event.preventDefault();
        setSelectedIndex(prev => Math.max(prev - 1, 0));
        break;
      case 'Enter':
        event.preventDefault();
        if (filteredHistory[selectedIndex]) {
          onCommandSelect(filteredHistory[selectedIndex].text);
          onClose();
        }
        break;
      case 'Escape':
        event.preventDefault();
        onClose();
        break;
    }
  };

  const formatTimestamp = (timestamp: number): string => {
    const date = new Date(timestamp);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffMins = Math.floor(diffMs / (1000 * 60));
    const diffHours = Math.floor(diffMins / 60);
    const diffDays = Math.floor(diffHours / 24);

    if (diffMins < 1) return 'just now';
    if (diffMins < 60) return `${diffMins}m ago`;
    if (diffHours < 24) return `${diffHours}h ago`;
    if (diffDays < 7) return `${diffDays}d ago`;
    return date.toLocaleDateString();
  };

  const formatDuration = (durationMs?: number): string => {
    if (!durationMs) return '';
    if (durationMs < 1000) return `${durationMs}ms`;
    const seconds = Math.floor(durationMs / 1000);
    if (seconds < 60) return `${seconds}s`;
    const minutes = Math.floor(seconds / 60);
    return `${minutes}m ${seconds % 60}s`;
  };

  if (!visible) {
    return null;
  }

  return (
    <div className="command-history-overlay">
      <div className="command-history-modal">
        <div className="history-header">
          <h3>Command History</h3>
          <button className="close-button" onClick={onClose}>×</button>
        </div>
        
        <div className="history-search">
          <input
            ref={searchRef}
            type="text"
            placeholder="Search commands..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            onKeyDown={handleKeyDown}
            className="search-input"
          />
        </div>

        <div className="history-content" ref={historyRef}>
          {loading ? (
            <div className="history-loading">Loading history...</div>
          ) : filteredHistory.length === 0 ? (
            <div className="history-empty">
              {searchQuery ? 'No commands found matching your search.' : 'No command history available.'}
            </div>
          ) : (
            <div className="history-list">
              {filteredHistory.map((command, index) => (
                <div
                  key={command.id}
                  className={`history-item ${index === selectedIndex ? 'selected' : ''}`}
                  onClick={() => {
                    onCommandSelect(command.text);
                    onClose();
                  }}
                >
                  <div className="history-command">{command.text}</div>
                  <div className="history-meta">
                    <span className="history-dir">{command.working_dir}</span>
                    <span className="history-time">{formatTimestamp(command.timestamp)}</span>
                    {command.duration_ms && (
                      <span className="history-duration">{formatDuration(command.duration_ms)}</span>
                    )}
                    {command.exit_code !== undefined && command.exit_code !== 0 && (
                      <span className="history-exit-code error">exit {command.exit_code}</span>
                    )}
                  </div>
                </div>
              ))}
            </div>
          )}
        </div>

        <div className="history-footer">
          <span className="history-hint">
            ↑↓ navigate • Enter select • Esc close
          </span>
        </div>
      </div>
    </div>
  );
};

export default CommandHistory;
