import React, { useState, useEffect, useRef } from 'react';
import { invoke } from '@tauri-apps/api/core';

interface CommandSuggestion {
  command: string;
  description: string;
  frequency: number;
  last_used: number;
}

interface CommandCompletionProps {
  terminalId: string;
  currentInput: string;
  cursorPosition: number;
  onSuggestionSelect: (suggestion: string) => void;
  visible: boolean;
  onClose: () => void;
}

const CommandCompletion: React.FC<CommandCompletionProps> = ({
  terminalId,
  currentInput,
  cursorPosition,
  onSuggestionSelect,
  visible,
  onClose,
}) => {
  const [suggestions, setSuggestions] = useState<CommandSuggestion[]>([]);
  const [selectedIndex, setSelectedIndex] = useState(0);
  const [loading, setLoading] = useState(false);
  const completionRef = useRef<HTMLDivElement>(null);

  // Get suggestions when input changes
  useEffect(() => {
    if (!visible || !currentInput.trim()) {
      setSuggestions([]);
      return;
    }

    const fetchSuggestions = async () => {
      setLoading(true);
      try {
        // Extract the current word being typed
        const words = currentInput.substring(0, cursorPosition).split(' ');
        const currentWord = words[words.length - 1] || '';
        
        if (words.length === 1) {
          // Command completion
          const commandSuggestions = await invoke<CommandSuggestion[]>('get_command_suggestions', {
            terminalId,
            partialCommand: currentWord,
          });
          setSuggestions(commandSuggestions);
        } else {
          // Tab completion for files/arguments
          const tabCompletions = await invoke<string[]>('handle_tab_completion', {
            terminalId,
            currentLine: currentInput,
            cursorPos: cursorPosition,
          });
          
          // Convert to CommandSuggestion format
          const tabSuggestions: CommandSuggestion[] = tabCompletions.map(completion => ({
            command: completion,
            description: 'File/directory completion',
            frequency: 0,
            last_used: 0,
          }));
          setSuggestions(tabSuggestions);
        }
        setSelectedIndex(0);
      } catch (error) {
        console.error('Failed to get suggestions:', error);
        setSuggestions([]);
      } finally {
        setLoading(false);
      }
    };

    const debounceTimer = setTimeout(fetchSuggestions, 150);
    return () => clearTimeout(debounceTimer);
  }, [terminalId, currentInput, cursorPosition, visible]);

  // Handle keyboard navigation
  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      if (!visible || suggestions.length === 0) return;

      switch (event.key) {
        case 'ArrowDown':
          event.preventDefault();
          setSelectedIndex(prev => Math.min(prev + 1, suggestions.length - 1));
          break;
        case 'ArrowUp':
          event.preventDefault();
          setSelectedIndex(prev => Math.max(prev - 1, 0));
          break;
        case 'Tab':
        case 'Enter':
          event.preventDefault();
          if (suggestions[selectedIndex]) {
            onSuggestionSelect(suggestions[selectedIndex].command);
          }
          break;
        case 'Escape':
          event.preventDefault();
          onClose();
          break;
      }
    };

    if (visible) {
      window.addEventListener('keydown', handleKeyDown);
      return () => window.removeEventListener('keydown', handleKeyDown);
    }
  }, [visible, suggestions, selectedIndex, onSuggestionSelect, onClose]);

  // Auto-scroll selected item into view
  useEffect(() => {
    if (completionRef.current && suggestions.length > 0) {
      const selectedElement = completionRef.current.children[selectedIndex] as HTMLElement;
      if (selectedElement) {
        selectedElement.scrollIntoView({ block: 'nearest' });
      }
    }
  }, [selectedIndex, suggestions]);

  if (!visible || suggestions.length === 0) {
    return null;
  }

  return (
    <div className="command-completion-overlay">
      <div ref={completionRef} className="command-completion-menu">
        <div className="completion-header">
          <span className="completion-title">Suggestions</span>
          {loading && <span className="completion-loading">⟳</span>}
        </div>
        <div className="completion-list">
          {suggestions.map((suggestion, index) => (
            <div
              key={index}
              className={`completion-item ${index === selectedIndex ? 'selected' : ''}`}
              onClick={() => onSuggestionSelect(suggestion.command)}
            >
              <div className="completion-command">{suggestion.command}</div>
              <div className="completion-description">{suggestion.description}</div>
              {suggestion.frequency > 1 && (
                <div className="completion-frequency">×{suggestion.frequency}</div>
              )}
            </div>
          ))}
        </div>
        <div className="completion-footer">
          <span className="completion-hint">
            ↑↓ navigate • Tab/Enter select • Esc close
          </span>
        </div>
      </div>
    </div>
  );
};

export default CommandCompletion;
